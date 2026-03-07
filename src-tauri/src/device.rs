use parking_lot::Mutex;
use rusb::{Context, DeviceHandle, UsbContext};
use std::sync::Arc;
use std::time::Duration;

use crate::protocol::*;

/// USB endpoints for K13 R2R interface 3
const INTERFACE: u8 = 3;
const EP_OUT: u8 = 0x02; // Interrupt OUT
const EP_IN: u8 = 0x83; // Interrupt IN
const REPORT_ID_K13: u8 = 7;
const TIMEOUT_WRITE: Duration = Duration::from_millis(1000);
const TIMEOUT_READ: Duration = Duration::from_millis(2000); // Chrome uses 2s

pub struct FiiODevice {
    handle: Option<DeviceHandle<Context>>,
    product_name: Option<String>,
    had_kernel_driver: bool,
}

impl FiiODevice {
    pub fn new() -> Self {
        Self {
            handle: None,
            product_name: None,
            had_kernel_driver: false,
        }
    }

    pub fn connect(&mut self) -> Result<String, String> {
        let context = Context::new().map_err(|e| format!("Failed to init USB context: {}", e))?;

        // Find K13 by vendor ID
        let devices = context.devices().map_err(|e| format!("Failed to list USB devices: {}", e))?;

        for device in devices.iter() {
            let desc = device.device_descriptor().map_err(|e| format!("descriptor: {}", e))?;
            if desc.vendor_id() != FIIO_VENDOR_ID {
                continue;
            }

            let handle = device.open().map_err(|e| format!("Failed to open device: {}", e))?;

            // Read product name
            let timeout = Duration::from_secs(1);
            let langs = handle.read_languages(timeout).unwrap_or_default();
            let name = if !langs.is_empty() {
                handle
                    .read_product_string(langs[0], &desc, timeout)
                    .unwrap_or_default()
            } else {
                String::new()
            };

            if !name.to_lowercase().contains("k13") {
                continue;
            }

            log::info!("Found K13: VID={:04x} PID={:04x} name={}", desc.vendor_id(), desc.product_id(), name);

            // Enable auto-detach so kernel driver is reattached on drop
            let _ = handle.set_auto_detach_kernel_driver(true);

            // Detach kernel driver from interface 3 if active
            let had_driver = match handle.kernel_driver_active(INTERFACE) {
                Ok(true) => {
                    handle.detach_kernel_driver(INTERFACE)
                        .map_err(|e| format!("Failed to detach kernel driver: {}", e))?;
                    log::info!("Detached kernel driver from interface {}", INTERFACE);
                    true
                }
                Ok(false) => false,
                Err(rusb::Error::NotSupported) => false,
                Err(e) => return Err(format!("Failed to check kernel driver: {}", e)),
            };

            // Claim interface 3
            handle.claim_interface(INTERFACE)
                .map_err(|e| format!("Failed to claim interface {}: {}", INTERFACE, e))?;
            log::info!("Claimed interface {}", INTERFACE);

            self.handle = Some(handle);
            self.product_name = Some(name.clone());
            self.had_kernel_driver = had_driver;
            return Ok(name);
        }

        Err("No FiiO K13 R2R found. Make sure it's connected via USB.".to_string())
    }

    pub fn disconnect(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.release_interface(INTERFACE);
            if self.had_kernel_driver {
                let _ = handle.attach_kernel_driver(INTERFACE);
            }
            log::info!("Disconnected");
        }
        self.product_name = None;
        self.had_kernel_driver = false;
    }

    pub fn is_connected(&self) -> bool {
        self.handle.is_some()
    }

    pub fn product_name(&self) -> Option<&str> {
        self.product_name.as_deref()
    }

    /// Send data via interrupt OUT with report ID prefix (exactly like Chrome sendReport)
    fn usb_write(&self, packet: &[u8]) -> Result<(), String> {
        let handle = self.handle.as_ref().ok_or("Device not connected")?;

        // Chrome: sendReport(7, data) — prepends report ID to 64-byte payload
        let mut buf = vec![0u8; 65]; // report ID + 64 bytes
        buf[0] = REPORT_ID_K13;
        let copy_len = packet.len().min(64);
        buf[1..1 + copy_len].copy_from_slice(&packet[..copy_len]);

        log::debug!("USB TX [EP {:02X}]: {:02X?}", EP_OUT, &buf[..copy_len.min(16) + 1]);

        handle
            .write_interrupt(EP_OUT, &buf, TIMEOUT_WRITE)
            .map_err(|e| format!("USB write failed: {}", e))?;

        Ok(())
    }

    /// Read one response from interrupt IN, stripping report ID
    fn usb_read_raw(&self, timeout: Duration) -> Result<Vec<u8>, String> {
        let handle = self.handle.as_ref().ok_or("Device not connected")?;

        let mut buf = [0u8; 65];
        let len = handle
            .read_interrupt(EP_IN, &mut buf, timeout)
            .map_err(|e| format!("USB read failed: {}", e))?;

        log::debug!("USB RX [EP {:02X}]: {} bytes: {:02X?}", EP_IN, len, &buf[..len.min(16)]);

        // Skip report ID byte if present (Chrome strips it)
        if len > 0 && buf[0] == REPORT_ID_K13 {
            Ok(buf[1..len].to_vec())
        } else {
            Ok(buf[..len].to_vec())
        }
    }

    /// Drain any stale responses sitting in the IN buffer
    fn drain_stale(&self) {
        let handle = match self.handle.as_ref() {
            Some(h) => h,
            None => return,
        };
        let mut buf = [0u8; 65];
        // Use longer timeout to catch in-flight responses from SET commands
        let timeout = Duration::from_millis(100);
        loop {
            match handle.read_interrupt(EP_IN, &mut buf, timeout) {
                Ok(len) => {
                    log::debug!("USB DRAIN: {} bytes: {:02X?}", len, &buf[..len.min(16)]);
                }
                Err(_) => break, // Timeout = buffer empty
            }
        }
    }

    /// Send command and read matching response (like Chrome sendReportAndListen)
    /// Matches response CMD byte to request CMD byte, retries on mismatch
    fn send_and_receive(&self, packet: &[u8]) -> Result<Vec<u8>, String> {
        self.drain_stale();
        self.usb_write(packet)?;

        let expected_cmd = if packet.len() > 4 { packet[4] } else { 0 };

        // Try up to 3 reads to find matching response
        for attempt in 0..3 {
            match self.usb_read_raw(TIMEOUT_READ) {
                Ok(resp) => {
                    if resp.len() > 4 && resp[0] == 0xBB && resp[4] == expected_cmd {
                        return Ok(resp);
                    }
                    // Response for different command — stale, try again
                    log::debug!(
                        "Response CMD {:02X} doesn't match expected {:02X} (attempt {})",
                        resp.get(4).unwrap_or(&0), expected_cmd, attempt + 1
                    );
                }
                Err(e) => {
                    if e.contains("Timeout") {
                        log::debug!("Read timeout for CMD {:02X} (attempt {})", expected_cmd, attempt + 1);
                        return Err(format!("Timeout reading response for CMD {:02X}", expected_cmd));
                    }
                    return Err(e);
                }
            }
        }

        Err(format!("No matching response for CMD {:02X} after 3 attempts", expected_cmd))
    }

    /// Send command without waiting for response (like Chrome sendReport for SET ops)
    fn send_only(&self, packet: &[u8]) -> Result<(), String> {
        self.usb_write(packet)?;
        // Wait for device to process before sending next command
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    // ---- EQ Operations ----

    pub fn get_eq_count(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_eq_count())?;
        // Response: [BB, 0B, 0, 0, CMD, LEN, count, ...]
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(10) // Default
        }
    }

    pub fn get_eq_band(&self, index: u8) -> Result<EqBand, String> {
        let resp = self.send_and_receive(&get_eq_band_item(index))?;
        // Response: [BB, 0B, 0, 0, 0x15, LEN, index, gain_hi, gain_lo, freq_hi, freq_lo, q_hi, q_lo, filter_type, ...]
        if resp.len() >= 14 && resp[0] == 0xBB {
            Ok(EqBand {
                index: resp[6],
                gain: parse_gain(resp[7], resp[8]),
                frequency: parse_frequency(resp[9], resp[10]),
                q_value: parse_q_value(resp[11], resp[12]),
                filter_type: resp[13],
            })
        } else {
            Ok(EqBand::default_for_index(index))
        }
    }

    pub fn get_all_eq_bands(&self) -> Result<Vec<EqBand>, String> {
        let count = self.get_eq_count()?;
        let mut bands = Vec::new();
        for i in 0..count {
            bands.push(self.get_eq_band(i)?);
        }
        Ok(bands)
    }

    pub fn set_eq_band(
        &self,
        index: u8,
        freq: u16,
        gain: f64,
        q: f64,
        filter_type: u8,
    ) -> Result<(), String> {
        self.send_only(&set_eq_band_item(index, freq, gain, q, filter_type))
    }

    pub fn get_eq_preset(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_eq_preset())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(160)
        }
    }

    pub fn set_eq_preset(&self, preset: u8) -> Result<(), String> {
        self.send_only(&set_eq_preset(preset))
    }

    pub fn get_eq_global_gain(&self) -> Result<f64, String> {
        let resp = self.send_and_receive(&get_eq_global_gain())?;
        if resp.len() >= 8 && resp[0] == 0xBB {
            Ok(parse_gain(resp[6], resp[7]))
        } else {
            Ok(0.0)
        }
    }

    pub fn set_eq_global_gain(&self, gain: f64) -> Result<(), String> {
        self.send_only(&set_eq_global_gain(gain))
    }

    pub fn get_eq_switch(&self) -> Result<bool, String> {
        let resp = self.send_and_receive(&get_eq_switch())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6] != 0)
        } else {
            Ok(false)
        }
    }

    pub fn set_eq_switch(&self, enabled: bool) -> Result<(), String> {
        self.send_only(&set_eq_switch(if enabled { 1 } else { 0 }))
    }

    pub fn save_eq(&self, preset: u8) -> Result<(), String> {
        // Drain any pending SET responses before save
        self.drain_stale();
        // Save uses sendReportAndListen in Chrome (expects response)
        self.usb_write(&set_eq_save(preset))?;
        // Wait for device to persist
        std::thread::sleep(Duration::from_millis(200));
        // Drain save response
        self.drain_stale();
        Ok(())
    }

    pub fn reset_eq(&self) -> Result<(), String> {
        self.send_only(&set_eq_reset())
    }

    // ---- Config Operations ----

    pub fn get_firmware_version(&self) -> Result<String, String> {
        let resp = self.send_and_receive(&get_firmware_version())?;
        // Response: [BB, 0B, 0, 0, 0x0B, LEN, ...version_bytes...]
        if resp.len() > 6 && resp[0] == 0xBB {
            let len = resp[5] as usize;
            let end = (6 + len).min(resp.len());
            let version = String::from_utf8_lossy(&resp[6..end]).to_string();
            Ok(version)
        } else {
            Ok("Unknown".to_string())
        }
    }

    pub fn get_vol_max(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_vol_max())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(120)
        }
    }

    pub fn set_vol_max(&self, val: u8) -> Result<(), String> {
        self.send_only(&set_vol_max(val))
    }

    pub fn get_vol_output(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_vol_output())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(30)
        }
    }

    pub fn set_vol_output(&self, val: u8) -> Result<(), String> {
        self.send_only(&set_vol_output(val))
    }

    pub fn get_vol_output_switch(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_vol_output_switch())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(0)
        }
    }

    pub fn set_vol_output_switch(&self, val: u8) -> Result<(), String> {
        self.send_only(&set_vol_output_switch(val))
    }

    pub fn get_mic_switch(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_mic_switch())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(0)
        }
    }

    pub fn set_mic_switch(&self, val: u8) -> Result<(), String> {
        self.send_only(&set_mic_switch(val))
    }

    pub fn get_mic_monitor_vol(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_mic_monitor_vol())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(0)
        }
    }

    pub fn set_mic_monitor_vol(&self, val: u8) -> Result<(), String> {
        self.send_only(&set_mic_monitor_vol(val))
    }

    pub fn get_screen_orientation(&self) -> Result<u8, String> {
        let resp = self.send_and_receive(&get_screen_orientation())?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(0)
        }
    }

    pub fn set_screen_orientation(&self, val: u8) -> Result<(), String> {
        self.send_only(&set_screen_orientation(val))
    }

    pub fn get_channel_balance(&self) -> Result<i8, String> {
        let resp = self.send_and_receive(&get_channel_balance())?;
        if resp.len() >= 8 && resp[0] == 0xBB {
            // Response has two bytes: [left, right]
            let left = resp[6] as i8;
            let right = resp[7] as i8;
            if left < 0 {
                Ok(left)
            } else if right > 0 {
                Ok(right)
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    pub fn set_channel_balance(&self, val: i8) -> Result<(), String> {
        self.send_only(&set_channel_balance(val))
    }

    pub fn get_preset_name(&self, index: u8) -> Result<String, String> {
        let resp = self.send_and_receive(&get_preset_name(index))?;
        if resp.len() > 7 && resp[0] == 0xBB {
            let len = resp[5] as usize;
            // First data byte is index, rest is name
            let end = (7 + len - 1).min(resp.len());
            let name = String::from_utf8_lossy(&resp[7..end]).trim_end_matches('\0').to_string();
            Ok(name)
        } else {
            Ok(String::new())
        }
    }

    pub fn set_preset_name(&self, index: u8, name: &str) -> Result<(), String> {
        self.send_only(&set_preset_name(index, name))
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
    pub fn default_for_index(index: u8) -> Self {
        const DEFAULT_FREQS: [u16; 10] = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
        Self {
            index,
            gain: 0.0,
            frequency: DEFAULT_FREQS.get(index as usize).copied().unwrap_or(1000),
            q_value: 1.41,
            filter_type: 0,
        }
    }
}

pub type SharedDevice = Arc<Mutex<FiiODevice>>;

pub fn create_shared_device() -> SharedDevice {
    Arc::new(Mutex::new(FiiODevice::new()))
}
