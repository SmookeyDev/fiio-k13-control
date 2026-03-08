use parking_lot::Mutex;
use rusb::{Context, DeviceHandle, UsbContext};
use std::sync::Arc;
use std::time::Duration;

use crate::error::{AppError, Result};
use crate::protocol::*;

const INTERFACE: u8 = 3;
const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x83;
const REPORT_ID: u8 = 7;
const TIMEOUT_WRITE: Duration = Duration::from_millis(1000);
const TIMEOUT_READ: Duration = Duration::from_millis(2000);

pub type SharedDevice = Arc<Mutex<FiiODevice>>;

pub fn create_shared_device() -> SharedDevice {
    Arc::new(Mutex::new(FiiODevice::new()))
}

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

    // ---- Connection ----

    pub fn connect(&mut self) -> Result<String> {
        let context = Context::new()?;
        let devices = context.devices()?;

        for device in devices.iter() {
            let desc = device.device_descriptor()?;
            if desc.vendor_id() != FIIO_VENDOR_ID {
                continue;
            }

            let handle = device.open()?;
            let timeout = Duration::from_secs(1);
            let langs = handle.read_languages(timeout).unwrap_or_default();
            let name = if !langs.is_empty() {
                handle.read_product_string(langs[0], &desc, timeout).unwrap_or_default()
            } else {
                String::new()
            };

            if !name.to_lowercase().contains("k13") {
                continue;
            }

            log::info!("Found K13: VID={:04x} PID={:04x} name={}", desc.vendor_id(), desc.product_id(), name);

            let _ = handle.set_auto_detach_kernel_driver(true);

            let had_driver = match handle.kernel_driver_active(INTERFACE) {
                Ok(true) => {
                    handle.detach_kernel_driver(INTERFACE)?;
                    log::info!("Detached kernel driver from interface {}", INTERFACE);
                    true
                }
                Ok(false) => false,
                Err(rusb::Error::NotSupported) => false,
                Err(e) => return Err(e.into()),
            };

            handle.claim_interface(INTERFACE)?;
            log::info!("Claimed interface {}", INTERFACE);

            self.handle = Some(handle);
            self.product_name = Some(name.clone());
            self.had_kernel_driver = had_driver;
            return Ok(name);
        }

        Err(AppError::DeviceNotFound)
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

    // ---- USB transport ----

    fn handle(&self) -> Result<&DeviceHandle<Context>> {
        self.handle.as_ref().ok_or(AppError::NotConnected)
    }

    fn usb_write(&self, packet: &[u8]) -> Result<()> {
        let handle = self.handle()?;
        let mut buf = vec![0u8; 65];
        buf[0] = REPORT_ID;
        let copy_len = packet.len().min(64);
        buf[1..1 + copy_len].copy_from_slice(&packet[..copy_len]);

        log::debug!("USB TX: {:02X?}", &buf[..copy_len.min(16) + 1]);
        handle.write_interrupt(EP_OUT, &buf, TIMEOUT_WRITE)?;
        Ok(())
    }

    fn usb_read(&self, timeout: Duration) -> Result<Vec<u8>> {
        let handle = self.handle()?;
        let mut buf = [0u8; 65];
        let len = handle.read_interrupt(EP_IN, &mut buf, timeout)?;

        log::debug!("USB RX: {} bytes: {:02X?}", len, &buf[..len.min(16)]);

        if len > 0 && buf[0] == REPORT_ID {
            Ok(buf[1..len].to_vec())
        } else {
            Ok(buf[..len].to_vec())
        }
    }

    fn drain_stale(&self) {
        let handle = match self.handle.as_ref() {
            Some(h) => h,
            None => return,
        };
        let mut buf = [0u8; 65];
        let timeout = Duration::from_millis(100);
        while handle.read_interrupt(EP_IN, &mut buf, timeout).is_ok() {}
    }

    fn send_and_receive(&self, packet: &[u8]) -> Result<Vec<u8>> {
        self.drain_stale();
        self.usb_write(packet)?;

        let expected_cmd = if packet.len() > 4 { packet[4] } else { 0 };

        for attempt in 0..3 {
            match self.usb_read(TIMEOUT_READ) {
                Ok(resp) => {
                    if resp.len() > 4 && resp[0] == 0xBB && resp[4] == expected_cmd {
                        return Ok(resp);
                    }
                    log::debug!(
                        "CMD mismatch: got {:02X}, expected {:02X} (attempt {})",
                        resp.get(4).unwrap_or(&0), expected_cmd, attempt + 1
                    );
                }
                Err(e) => return Err(e),
            }
        }

        Err(AppError::Protocol(format!(
            "No matching response for CMD {:02X} after 3 attempts", expected_cmd
        )))
    }

    fn send_only(&self, packet: &[u8]) -> Result<()> {
        self.drain_stale();
        self.usb_write(packet)?;
        std::thread::sleep(Duration::from_millis(50));
        self.drain_stale();
        Ok(())
    }

    /// Read a single u8 value from a GET response at byte offset 6
    fn read_u8(&self, packet: &[u8], default: u8) -> Result<u8> {
        let resp = self.send_and_receive(packet)?;
        if resp.len() > 6 && resp[0] == 0xBB {
            Ok(resp[6])
        } else {
            Ok(default)
        }
    }

    // ---- EQ Operations ----

    pub fn get_eq_count(&self) -> Result<u8> {
        self.read_u8(&get_eq_count(), 10)
    }

    pub fn get_eq_band(&self, index: u8) -> Result<EqBand> {
        let resp = self.send_and_receive(&get_eq_band_item(index))?;
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

    pub fn get_all_eq_bands(&self) -> Result<Vec<EqBand>> {
        let count = self.get_eq_count()?;
        (0..count).map(|i| self.get_eq_band(i)).collect()
    }

    pub fn set_eq_band(&self, index: u8, freq: u16, gain: f64, q: f64, filter_type: u8) -> Result<()> {
        self.send_only(&set_eq_band_item(index, freq, gain, q, filter_type))
    }

    pub fn get_eq_preset(&self) -> Result<u8> {
        self.read_u8(&get_eq_preset(), 160)
    }

    pub fn set_eq_preset(&self, preset: u8) -> Result<()> {
        self.send_only(&set_eq_preset(preset))
    }

    pub fn get_eq_global_gain(&self) -> Result<f64> {
        let resp = self.send_and_receive(&get_eq_global_gain())?;
        if resp.len() >= 8 && resp[0] == 0xBB {
            Ok(parse_gain(resp[6], resp[7]))
        } else {
            Ok(0.0)
        }
    }

    pub fn set_eq_global_gain(&self, gain: f64) -> Result<()> {
        self.send_only(&set_eq_global_gain(gain))
    }

    pub fn get_eq_switch(&self) -> Result<bool> {
        Ok(self.read_u8(&get_eq_switch(), 0)? != 0)
    }

    pub fn set_eq_switch(&self, enabled: bool) -> Result<()> {
        self.send_only(&set_eq_switch(if enabled { 1 } else { 0 }))
    }

    pub fn save_eq(&self, preset: u8) -> Result<()> {
        self.drain_stale();
        self.usb_write(&set_eq_save(preset))?;
        std::thread::sleep(Duration::from_millis(200));
        self.drain_stale();
        Ok(())
    }

    pub fn reset_eq(&self) -> Result<()> {
        self.send_only(&set_eq_reset())
    }

    // ---- Preset Names ----

    pub fn get_preset_name(&self, index: u8) -> Result<String> {
        let resp = self.send_and_receive(&get_preset_name(index))?;
        if resp.len() > 7 && resp[0] == 0xBB {
            let len = resp[5] as usize;
            let end = (7 + len - 1).min(resp.len());
            let name = String::from_utf8_lossy(&resp[7..end]).trim_end_matches('\0').to_string();
            Ok(name)
        } else {
            Ok(String::new())
        }
    }

    pub fn set_preset_name(&self, index: u8, name: &str) -> Result<()> {
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
