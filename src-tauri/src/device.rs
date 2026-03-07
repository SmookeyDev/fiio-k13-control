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

    /// Send a command to the device via hidraw write().
    /// The K13 accepts data via interrupt OUT endpoint.
    /// Responses CANNOT be read on Linux due to the device's broken HID descriptor
    /// (declares only Feature reports, but doesn't actually support them).
    fn hid_write(&self, packet: &[u8]) -> Result<(), String> {
        let dev = self.device.as_ref().ok_or("Device not connected")?;

        // Pad to 64 bytes — the K13 has no numbered reports
        let mut buf = [0u8; 64];
        let copy_len = packet.len().min(64);
        buf[..copy_len].copy_from_slice(&packet[..copy_len]);

        log::debug!("HID TX: {:02X?}", &buf[..copy_len.min(16)]);

        dev.write(&buf)
            .map_err(|e| format!("HID write failed: {}", e))?;

        // Device needs time to process
        std::thread::sleep(Duration::from_millis(30));
        Ok(())
    }

    // ---- EQ Operations ----

    pub fn get_eq_count(&self) -> Result<u8, String> {
        // K13 R2R has 10 EQ bands (fixed, from web app analysis)
        Ok(10)
    }

    pub fn get_eq_band(&self, index: u8) -> Result<EqBand, String> {
        // Cannot read from device on Linux — return default band
        Ok(EqBand::default_for_index(index))
    }

    pub fn get_all_eq_bands(&self) -> Result<Vec<EqBand>, String> {
        // Cannot read from device on Linux — return 10 default bands
        Ok((0..10).map(EqBand::default_for_index).collect())
    }

    pub fn set_eq_band(
        &self,
        index: u8,
        freq: u16,
        gain: f64,
        q: f64,
        filter_type: u8,
    ) -> Result<(), String> {
        self.hid_write(&set_eq_band_item(index, freq, gain, q, filter_type))
    }

    pub fn get_eq_preset(&self) -> Result<u8, String> {
        Ok(160) // Cannot read — default USER 1
    }

    pub fn set_eq_preset(&self, preset: u8) -> Result<(), String> {
        self.hid_write(&set_eq_preset(preset))
    }

    pub fn get_eq_global_gain(&self) -> Result<f64, String> {
        Ok(0.0) // Cannot read
    }

    pub fn set_eq_global_gain(&self, gain: f64) -> Result<(), String> {
        self.hid_write(&set_eq_global_gain(gain))
    }

    pub fn get_eq_switch(&self) -> Result<bool, String> {
        Ok(false) // Cannot read
    }

    pub fn set_eq_switch(&self, enabled: bool) -> Result<(), String> {
        self.hid_write(&set_eq_switch(if enabled { 1 } else { 0 }))
    }

    pub fn save_eq(&self, preset: u8) -> Result<(), String> {
        self.hid_write(&set_eq_save(preset))
    }

    pub fn reset_eq(&self) -> Result<(), String> {
        self.hid_write(&set_eq_reset())
    }

    // ---- Config Operations ----

    pub fn get_firmware_version(&self) -> Result<String, String> {
        Ok("Unknown (Linux)".to_string()) // Cannot read
    }

    pub fn get_vol_max(&self) -> Result<u8, String> {
        Ok(120) // Default max volume
    }

    pub fn set_vol_max(&self, val: u8) -> Result<(), String> {
        self.hid_write(&set_vol_max(val))
    }

    pub fn get_vol_output(&self) -> Result<u8, String> {
        Ok(30) // Default volume
    }

    pub fn set_vol_output(&self, val: u8) -> Result<(), String> {
        self.hid_write(&set_vol_output(val))
    }

    pub fn get_vol_output_switch(&self) -> Result<u8, String> {
        Ok(0) // Default
    }

    pub fn set_vol_output_switch(&self, val: u8) -> Result<(), String> {
        self.hid_write(&set_vol_output_switch(val))
    }

    pub fn get_mic_switch(&self) -> Result<u8, String> {
        Ok(0)
    }

    pub fn set_mic_switch(&self, val: u8) -> Result<(), String> {
        self.hid_write(&set_mic_switch(val))
    }

    pub fn get_mic_monitor_vol(&self) -> Result<u8, String> {
        Ok(0)
    }

    pub fn set_mic_monitor_vol(&self, val: u8) -> Result<(), String> {
        self.hid_write(&set_mic_monitor_vol(val))
    }

    pub fn get_screen_orientation(&self) -> Result<u8, String> {
        Ok(0)
    }

    pub fn set_screen_orientation(&self, val: u8) -> Result<(), String> {
        self.hid_write(&set_screen_orientation(val))
    }

    pub fn get_channel_balance(&self) -> Result<i8, String> {
        Ok(0)
    }

    pub fn set_channel_balance(&self, val: i8) -> Result<(), String> {
        self.hid_write(&set_channel_balance(val))
    }

    pub fn get_preset_name(&self, _index: u8) -> Result<String, String> {
        Ok(String::new()) // Cannot read
    }

    pub fn set_preset_name(&self, index: u8, name: &str) -> Result<(), String> {
        self.hid_write(&set_preset_name(index, name))
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

impl EqBand {
    /// Default bands matching K13 R2R factory defaults
    pub fn default_for_index(index: u8) -> Self {
        const DEFAULT_FREQS: [u16; 10] = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
        Self {
            index,
            gain: 0.0,
            frequency: DEFAULT_FREQS.get(index as usize).copied().unwrap_or(1000),
            q_value: 1.41,
            filter_type: 0, // Peak
        }
    }
}

pub type SharedDevice = Arc<Mutex<FiiODevice>>;

pub fn create_shared_device() -> SharedDevice {
    Arc::new(Mutex::new(FiiODevice::new()))
}
