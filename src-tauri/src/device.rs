use hidapi::{HidApi, HidDevice};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

use crate::protocol::*;

pub struct FiiODevice {
    device: Option<HidDevice>,
    product_name: Option<String>,
}

impl FiiODevice {
    pub fn new() -> Self {
        Self {
            device: None,
            product_name: None,
        }
    }

    pub fn connect(&mut self) -> Result<String, String> {
        let api = HidApi::new().map_err(|e| format!("Failed to init HID API: {}", e))?;

        for dev_info in api.device_list() {
            if dev_info.vendor_id() == FIIO_VENDOR_ID {
                log::info!(
                    "Found FiiO HID: name={:?} vid={} pid={} iface={} path={:?}",
                    dev_info.product_string(),
                    dev_info.vendor_id(),
                    dev_info.product_id(),
                    dev_info.interface_number(),
                    dev_info.path(),
                );
            }
        }

        // Find K13 HID interfaces, prefer interface 3
        let mut k13_interfaces: Vec<_> = api
            .device_list()
            .filter(|d| d.vendor_id() == FIIO_VENDOR_ID)
            .filter(|d| {
                let name = d.product_string().unwrap_or("");
                name.contains("K13") || name.contains("k13")
            })
            .collect();

        k13_interfaces.sort_by_key(|d| if d.interface_number() == 3 { 0 } else { 1 });

        if let Some(dev_info) = k13_interfaces.first() {
            let name = dev_info
                .product_string()
                .unwrap_or("FIIO K13 R2R")
                .to_string();
            log::info!(
                "Opening K13: iface={} path={:?}",
                dev_info.interface_number(),
                dev_info.path()
            );
            let device = dev_info
                .open_device(&api)
                .map_err(|e| format!("Failed to open device: {}", e))?;

            // hidraw backend: non-blocking by default, we use read_timeout
            self.product_name = Some(name.clone());
            self.device = Some(device);
            return Ok(name);
        }

        Err("No FiiO device found. Make sure your K13 R2R is connected via USB.".to_string())
    }

    pub fn disconnect(&mut self) {
        self.device = None;
        self.product_name = None;
    }

    pub fn is_connected(&self) -> bool {
        self.device.is_some()
    }

    pub fn product_name(&self) -> Option<&str> {
        self.product_name.as_deref()
    }

    /// Send a command via Feature report and read response via Feature report.
    /// The K13's HID descriptor only defines Feature reports (64 bytes, no report ID).
    /// hidraw's write() sends SET_REPORT, and get_feature_report() sends GET_REPORT.
    fn send_and_receive(&self, packet: &[u8]) -> Result<Vec<u8>, String> {
        let dev = self.device.as_ref().ok_or("Device not connected")?;

        // Pad packet to 64 bytes for the HID report
        let mut write_buf = [0u8; 65]; // byte 0 = report ID (0 = no report ID)
        // write_buf[0] = 0; // report ID 0 (descriptor has no report ID)
        let copy_len = packet.len().min(64);
        write_buf[1..1 + copy_len].copy_from_slice(&packet[..copy_len]);

        log::debug!(
            "HID TX ({} bytes): {:02X?}",
            copy_len,
            &write_buf[1..1 + copy_len.min(16)]
        );

        // Send via write() which goes through hidraw → SET_REPORT
        dev.write(&write_buf)
            .map_err(|e| format!("HID write failed: {}", e))?;

        // Small delay for device to process command
        std::thread::sleep(Duration::from_millis(50));

        // Read response via Feature report (GET_REPORT)
        let mut resp_buf = [0u8; 65]; // byte 0 = report ID
        match dev.get_feature_report(&mut resp_buf) {
            Ok(n) if n > 1 => {
                let data = &resp_buf[1..n]; // skip report ID byte
                log::debug!("HID RX ({} bytes): {:02X?}", n - 1, &data[..data.len().min(24)]);
                Ok(data.to_vec())
            }
            Ok(n) => {
                log::debug!("HID RX: too short ({})", n);
                Err("Empty response from device".to_string())
            }
            Err(e) => {
                log::warn!("HID read failed: {}", e);
                Err(format!("HID read failed: {}", e))
            }
        }
    }

    fn send_only(&self, packet: &[u8]) -> Result<(), String> {
        let dev = self.device.as_ref().ok_or("Device not connected")?;

        let mut write_buf = [0u8; 65];
        let copy_len = packet.len().min(64);
        write_buf[1..1 + copy_len].copy_from_slice(&packet[..copy_len]);

        log::debug!(
            "HID TX-only ({} bytes): {:02X?}",
            copy_len,
            &write_buf[1..1 + copy_len.min(16)]
        );

        dev.write(&write_buf)
            .map_err(|e| format!("HID write failed: {}", e))?;

        std::thread::sleep(Duration::from_millis(30));
        Ok(())
    }

    // ---- EQ Operations ----

    pub fn get_eq_count(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_eq_count())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err(format!("Invalid EQ count response (len={})", resp.len()))
        }
    }

    pub fn get_eq_band(&self, index: u8) -> Result<EqBand, String> {
        let resp = self.send_and_receive(&get_eq_band_item(index))?;
        if resp.len() < 14 {
            return Err(format!("Invalid EQ band response (len={})", resp.len()));
        }
        Ok(EqBand {
            index: resp[6],
            gain: parse_gain(resp[7], resp[8]),
            frequency: parse_frequency(resp[9], resp[10]),
            q_value: parse_q_value(resp[11], resp[12]),
            filter_type: resp[13],
        })
    }

    pub fn get_all_eq_bands(&self) -> Result<Vec<EqBand>, String> {
        let count = self.get_eq_count()?;
        let mut bands = Vec::with_capacity(count as usize);
        for i in 0..count {
            match self.get_eq_band(i) {
                Ok(band) => bands.push(band),
                Err(e) => {
                    log::warn!("Failed to get EQ band {}: {}", i, e);
                    bands.push(EqBand {
                        index: i,
                        gain: 0.0,
                        frequency: 20,
                        q_value: 1.0,
                        filter_type: 0,
                    });
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(bands)
    }

    pub fn set_eq_band(&self, index: u8, freq: u16, gain: f64, q: f64, filter_type: u8) -> Result<(), String> {
        self.send_only(&set_eq_band_item(index, freq, gain, q, filter_type))
    }

    pub fn get_eq_preset(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_eq_preset())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err("Invalid EQ preset response".to_string())
        }
    }

    pub fn set_eq_preset(&self, preset: u8) -> Result<(), String> {
        self.send_and_receive(&set_eq_preset(preset))?;
        Ok(())
    }

    pub fn get_eq_global_gain(&self) -> Result<f64, String> {
        let resp = self.send_and_receive(&get_eq_global_gain())?;
        if resp.len() >= 8 {
            Ok(parse_gain(resp[6], resp[7]))
        } else {
            Err("Invalid EQ global gain response".to_string())
        }
    }

    pub fn set_eq_global_gain(&self, gain: f64) -> Result<(), String> {
        self.send_and_receive(&set_eq_global_gain(gain))?;
        Ok(())
    }

    pub fn get_eq_switch(&self) -> Result<bool, String> {
        let resp = self.send_and_receive(&get_eq_switch())?;
        if resp.len() >= 7 {
            Ok(resp[6] != 0)
        } else {
            Err("Invalid EQ switch response".to_string())
        }
    }

    pub fn set_eq_switch(&self, enabled: bool) -> Result<(), String> {
        self.send_and_receive(&set_eq_switch(if enabled { 1 } else { 0 }))?;
        Ok(())
    }

    pub fn save_eq(&self, preset: u8) -> Result<(), String> {
        self.send_and_receive(&set_eq_save(preset))?;
        Ok(())
    }

    pub fn reset_eq(&self) -> Result<(), String> {
        self.send_and_receive(&set_eq_reset())?;
        Ok(())
    }

    // ---- Config Operations ----

    pub fn get_firmware_version(&self) -> Result<String, String> {
        let resp = self.send_and_receive(&get_firmware_version())?;
        log::info!(
            "Firmware response ({} bytes): {:02X?}",
            resp.len(),
            &resp[..resp.len().min(24)]
        );
        if resp.len() >= 7 {
            let len = resp[5] as usize;
            let end = (6 + len).min(resp.len());
            let version_bytes = &resp[6..end];
            let version = String::from_utf8_lossy(version_bytes).to_string();
            log::info!(
                "Firmware: '{}' (data_len={}, raw={:02X?})",
                version,
                len,
                version_bytes
            );
            Ok(version)
        } else {
            Err(format!(
                "Firmware: response too short (len={})",
                resp.len()
            ))
        }
    }

    pub fn get_vol_max(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_vol_max())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err("Invalid vol max response".to_string())
        }
    }

    pub fn set_vol_max(&self, val: u8) -> Result<(), String> {
        self.send_and_receive(&set_vol_max(val))?;
        Ok(())
    }

    pub fn get_vol_output(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_vol_output())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err("Invalid response".to_string())
        }
    }

    pub fn set_vol_output(&self, val: u8) -> Result<(), String> {
        self.send_and_receive(&set_vol_output(val))?;
        Ok(())
    }

    pub fn get_vol_output_switch(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_vol_output_switch())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err("Invalid response".to_string())
        }
    }

    pub fn set_vol_output_switch(&self, val: u8) -> Result<(), String> {
        self.send_and_receive(&set_vol_output_switch(val))?;
        Ok(())
    }

    pub fn get_mic_switch(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_mic_switch())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err("Invalid response".to_string())
        }
    }

    pub fn set_mic_switch(&self, val: u8) -> Result<(), String> {
        self.send_and_receive(&set_mic_switch(val))?;
        Ok(())
    }

    pub fn get_mic_monitor_vol(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_mic_monitor_vol())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err("Invalid response".to_string())
        }
    }

    pub fn set_mic_monitor_vol(&self, val: u8) -> Result<(), String> {
        self.send_and_receive(&set_mic_monitor_vol(val))?;
        Ok(())
    }

    pub fn get_screen_orientation(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_screen_orientation())?;
        if resp.len() >= 7 {
            Ok(resp[6])
        } else {
            Err("Invalid response".to_string())
        }
    }

    pub fn set_screen_orientation(&self, val: u8) -> Result<(), String> {
        self.send_and_receive(&set_screen_orientation(val))?;
        Ok(())
    }

    pub fn get_channel_balance(&self) -> Result<i8, String> {
        let resp = self.send_and_receive(&get_channel_balance())?;
        if resp.len() >= 8 {
            let left = resp[6] as i8;
            let right = resp[7] as i8;
            if left != 0 {
                Ok(left)
            } else {
                Ok(right)
            }
        } else {
            Err("Invalid response".to_string())
        }
    }

    pub fn set_channel_balance(&self, val: i8) -> Result<(), String> {
        self.send_and_receive(&set_channel_balance(val))?;
        Ok(())
    }

    pub fn get_preset_name(&self, index: u8) -> Result<String, String> {
        let resp = self.send_and_receive(&get_preset_name(index))?;
        if resp.len() >= 7 {
            let len = resp[5] as usize;
            let end = (6 + len).min(resp.len());
            let name_data = &resp[7..end]; // skip the index byte at [6]
            let name = String::from_utf8_lossy(name_data)
                .trim_end_matches('\0')
                .to_string();
            Ok(name)
        } else {
            Ok(String::new())
        }
    }

    pub fn set_preset_name(&self, index: u8, name: &str) -> Result<(), String> {
        self.send_and_receive(&set_preset_name(index, name))?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EqBand {
    pub index: u8,
    pub gain: f64,
    pub frequency: u16,
    pub q_value: f64,
    pub filter_type: u8,
}

pub type SharedDevice = Arc<Mutex<FiiODevice>>;

pub fn create_shared_device() -> SharedDevice {
    Arc::new(Mutex::new(FiiODevice::new()))
}
