use btleplug::api::{
    Central, Manager as _, Peripheral as _, ScanFilter, WriteType,
    Characteristic,
};
use btleplug::platform::{Manager, Peripheral};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

use crate::ble_protocol;
use crate::ble_protocol::*;
use crate::error::{AppError, Result};

pub type SharedBleDevice = Arc<Mutex<BleDevice>>;

pub fn create_shared_ble_device() -> SharedBleDevice {
    Arc::new(Mutex::new(BleDevice::new()))
}

pub struct BleDevice {
    peripheral: Option<Peripheral>,
    device_name: Option<String>,
    write_char: Option<Characteristic>,
    notify_char: Option<Characteristic>,
    response: Arc<Mutex<Option<Vec<u8>>>>,
    response_notify: Arc<Notify>,
}

impl BleDevice {
    pub fn new() -> Self {
        Self {
            peripheral: None,
            device_name: None,
            write_char: None,
            notify_char: None,
            response: Arc::new(Mutex::new(None)),
            response_notify: Arc::new(Notify::new()),
        }
    }

    /// Well-known Bluetooth Classic (BR/EDR) service UUIDs.
    const CLASSIC_BT_UUIDS: &[&str] = &[
        "0000110a-0000-1000-8000-00805f9b34fb",
        "0000110b-0000-1000-8000-00805f9b34fb",
        "0000110c-0000-1000-8000-00805f9b34fb",
        "0000110d-0000-1000-8000-00805f9b34fb",
        "0000110e-0000-1000-8000-00805f9b34fb",
        "00001112-0000-1000-8000-00805f9b34fb",
        "0000111e-0000-1000-8000-00805f9b34fb",
        "0000111f-0000-1000-8000-00805f9b34fb",
    ];

    fn is_classic_only(services: &[Uuid]) -> bool {
        if services.is_empty() {
            return false;
        }
        let classic: Vec<Uuid> = Self::CLASSIC_BT_UUIDS.iter()
            .map(|s| Uuid::parse_str(s).unwrap())
            .collect();
        services.iter().all(|s| classic.contains(s))
    }

    /// Scan via D-Bus with LE transport filter, then connect via D-Bus.
    /// btleplug hardcodes Transport::Auto in its scan which fails to find
    /// BLE devices on Intel AX210 in ControllerMode=le.
    fn dbus_scan_connect_disconnect(connect: bool) {
        // Set LE filter + start discovery
        let _ = std::process::Command::new("dbus-send")
            .args(["--system", "--print-reply", "--dest=org.bluez",
                   "--type=method_call", "/org/bluez/hci0",
                   "org.bluez.Adapter1.SetDiscoveryFilter",
                   "dict:string:variant:Transport,string:le"])
            .output();
        let _ = std::process::Command::new("dbus-send")
            .args(["--system", "--print-reply", "--dest=org.bluez",
                   "--type=method_call", "/org/bluez/hci0",
                   "org.bluez.Adapter1.StartDiscovery"])
            .output();
        std::thread::sleep(std::time::Duration::from_secs(6));
        let _ = std::process::Command::new("dbus-send")
            .args(["--system", "--print-reply", "--dest=org.bluez",
                   "--type=method_call", "/org/bluez/hci0",
                   "org.bluez.Adapter1.StopDiscovery"])
            .output();

        if !connect {
            return;
        }

        // Connect via bluetoothctl (maintains connection state unlike dbus-send)
        let _ = std::process::Command::new("bluetoothctl")
            .args(["--", "connect", "41:42:F9:EB:E1:35"])
            .output();
    }

    pub async fn connect(&mut self) -> Result<String> {
        const MAX_ATTEMPTS: u32 = 3;
        let mut last_err = String::new();

        for attempt in 1..=MAX_ATTEMPTS {
            log::info!("BLE: Connection attempt {attempt}/{MAX_ATTEMPTS}...");
            match self.try_connect().await {
                Ok(target) => return self.setup_notifications(target).await,
                Err(e) => {
                    last_err = format!("{e}");
                    log::warn!("BLE: Attempt {attempt} failed: {last_err}");
                    if attempt < MAX_ATTEMPTS {
                        // Full cleanup: remove device to clear BlueZ GATT cache,
                        // otherwise stale service discovery results persist across retries.
                        let _ = std::process::Command::new("bluetoothctl")
                            .args(["--", "disconnect", "41:42:F9:EB:E1:35"])
                            .output();
                        let _ = std::process::Command::new("bluetoothctl")
                            .args(["--", "remove", "41:42:F9:EB:E1:35"])
                            .output();
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                }
            }
        }

        Err(AppError::Ble(format!("BLE connect failed after {MAX_ATTEMPTS} attempts: {last_err}")))
    }

    async fn try_connect(&self) -> Result<Peripheral> {
        // Step 1: Scan and connect via D-Bus (handles LE filter correctly)
        log::info!("BLE: Scanning via D-Bus (LE filter)...");
        tokio::task::spawn_blocking(|| Self::dbus_scan_connect_disconnect(true))
            .await
            .map_err(|e| AppError::Ble(format!("D-Bus scan task: {e}")))?;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Step 2: Use btleplug to find the peripheral
        let manager = Manager::new().await
            .map_err(|e| AppError::Ble(format!("BLE manager: {e}")))?;

        let adapters = manager.adapters().await
            .map_err(|e| AppError::Ble(format!("BLE adapters: {e}")))?;

        let adapter = adapters.into_iter().next()
            .ok_or_else(|| AppError::Ble("No Bluetooth adapter found".into()))?;

        log::info!("BLE: Populating btleplug cache...");
        adapter.start_scan(ScanFilter::default()).await
            .map_err(|e| AppError::Ble(format!("BLE scan: {e}")))?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let _ = adapter.stop_scan().await;

        let peripherals = adapter.peripherals().await
            .map_err(|e| AppError::Ble(format!("BLE peripherals: {e}")))?;

        log::info!("BLE: Found {} peripherals in btleplug", peripherals.len());

        let target = self.find_k13(&peripherals).await?;

        // Check if already connected from D-Bus
        let connected = target.is_connected().await.unwrap_or(false);
        if connected {
            log::info!("BLE: Already connected via D-Bus");
        } else {
            log::info!("BLE: Connecting via btleplug...");
            let result = tokio::time::timeout(Duration::from_secs(10), target.connect()).await;
            match result {
                Ok(Ok(())) => log::info!("BLE: Connected via btleplug"),
                Ok(Err(e)) => return Err(AppError::Ble(format!("BLE connect: {e}"))),
                Err(_) => return Err(AppError::Ble("BLE connect timed out".into())),
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Step 3: Discover services and verify our characteristic exists
        log::info!("BLE: Discovering services...");
        target.discover_services().await
            .map_err(|e| AppError::Ble(format!("BLE discover: {e}")))?;

        let service_uuid = Uuid::parse_str(SERVICE_UUID).unwrap();
        let has_service = target.services().iter().any(|s| s.uuid == service_uuid);

        if !has_service {
            return Err(AppError::Ble("K13 service not found — will retry".into()));
        }

        for service in target.services() {
            log::info!("BLE: Service {:?}, chars: {:?}",
                service.uuid,
                service.characteristics.iter().map(|c| c.uuid).collect::<Vec<_>>());
        }

        Ok(target)
    }

    async fn setup_notifications(&mut self, target: Peripheral) -> Result<String> {
        let service_uuid = Uuid::parse_str(SERVICE_UUID).unwrap();
        let write_uuid = Uuid::parse_str(WRITE_UUID).unwrap();
        let notify_uuid = Uuid::parse_str(NOTIFY_UUID).unwrap();

        let mut write_char = None;
        let mut notify_char = None;

        for service in target.services() {
            if service.uuid != service_uuid {
                continue;
            }
            for ch in &service.characteristics {
                if ch.uuid == write_uuid {
                    write_char = Some(ch.clone());
                } else if ch.uuid == notify_uuid {
                    notify_char = Some(ch.clone());
                }
            }
        }

        let write_char = write_char
            .ok_or_else(|| AppError::Ble("Write characteristic not found".into()))?;
        let notify_char = notify_char
            .ok_or_else(|| AppError::Ble("Notify characteristic not found".into()))?;

        let response = self.response.clone();
        let notify = self.response_notify.clone();

        target.subscribe(&notify_char).await
            .map_err(|e| AppError::Ble(format!("BLE subscribe: {e}")))?;

        let peripheral_clone = target.clone();
        let notify_char_clone = notify_char.clone();
        tokio::spawn(async move {
            use btleplug::api::Peripheral as _;
            use futures::StreamExt;

            if let Ok(mut stream) = peripheral_clone.notifications().await {
                while let Some(data) = stream.next().await {
                    if data.uuid == notify_char_clone.uuid {
                        log::debug!("BLE RX: {} bytes: {:02X?}", data.value.len(), data.value);
                        let mut resp = response.lock().await;
                        *resp = Some(data.value);
                        drop(resp);
                        notify.notify_one();
                    }
                }
            }
        });

        tokio::time::sleep(Duration::from_millis(300)).await;

        let name = self.get_device_name_from_peripheral(&target).await;
        log::info!("BLE: Connected to {}", name);

        self.peripheral = Some(target);
        self.device_name = Some(name.clone());
        self.write_char = Some(write_char);
        self.notify_char = Some(notify_char);

        Ok(name)
    }

    pub async fn disconnect(&mut self) {
        if let Some(peripheral) = self.peripheral.take() {
            if let Some(char) = &self.notify_char {
                let _ = peripheral.unsubscribe(char).await;
            }
            let _ = peripheral.disconnect().await;
            log::info!("BLE: Disconnected");
        }
        self.device_name = None;
        self.write_char = None;
        self.notify_char = None;
    }

    pub fn is_connected(&self) -> bool {
        self.peripheral.is_some()
    }

    /// Check if the BLE connection is actually alive, not just stored.
    pub async fn check_connected(&mut self) -> bool {
        if let Some(ref p) = self.peripheral {
            if p.is_connected().await.unwrap_or(false) {
                return true;
            }
            // Peripheral exists but disconnected — clean up
            log::warn!("BLE: Connection lost (silent disconnect), cleaning up");
            self.force_cleanup().await;
        }
        false
    }

    /// Force cleanup of all BLE state without trying to gracefully disconnect.
    async fn force_cleanup(&mut self) {
        self.peripheral = None;
        self.device_name = None;
        self.write_char = None;
        self.notify_char = None;
        let mut resp = self.response.lock().await;
        *resp = None;
    }

    pub fn device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    async fn find_k13(&self, peripherals: &[Peripheral]) -> Result<Peripheral> {
        let service_uuid = Uuid::parse_str(SERVICE_UUID).unwrap();

        for p in peripherals {
            if let Ok(Some(props)) = p.properties().await {
                if props.services.contains(&service_uuid) {
                    let name = props.local_name.clone().unwrap_or_default();
                    log::info!("BLE: Found by service UUID: {} (addr: {:?})", name, props.address);
                    return Ok(p.clone());
                }
            }
        }

        for p in peripherals {
            if let Ok(Some(props)) = p.properties().await {
                let name = props.local_name.clone().unwrap_or_default();
                let name_lower = name.to_lowercase();
                if (name_lower.contains("k13") || name_lower.contains("fiio"))
                    && !Self::is_classic_only(&props.services)
                {
                    log::info!("BLE: Found by name: {} (addr: {:?}, services: {:?})",
                        name, props.address, props.services);
                    return Ok(p.clone());
                }
            }
        }

        for p in peripherals {
            if let Ok(Some(props)) = p.properties().await {
                let name = props.local_name.clone().unwrap_or_else(|| "???".into());
                log::debug!("BLE scan: {} addr={:?} services={:?}",
                    name, props.address, props.services);
            }
        }

        Err(AppError::Ble("K13 R2R not found via BLE. Is Bluetooth enabled and device in range?".into()))
    }

    async fn get_device_name_from_peripheral(&self, p: &Peripheral) -> String {
        if let Ok(Some(props)) = p.properties().await {
            props.local_name.unwrap_or_else(|| "FiiO K13 R2R".into())
        } else {
            "FiiO K13 R2R".into()
        }
    }

    /// Drain any pending notifications to prevent stale data from desync'ing.
    async fn drain_notifications(&self) {
        loop {
            let has_pending = {
                let resp = self.response.lock().await;
                resp.is_some()
            };
            if has_pending {
                let mut resp = self.response.lock().await;
                *resp = None;
            }
            // Try consuming any queued notify signals (non-blocking)
            match tokio::time::timeout(Duration::from_millis(50), self.response_notify.notified()).await {
                Ok(_) => continue, // consumed one, check for more
                Err(_) => break,   // no more pending
            }
        }
    }

    async fn send_and_receive(&self, packet: &[u8]) -> Result<Vec<u8>> {
        let peripheral = self.peripheral.as_ref().ok_or(AppError::NotConnected)?;
        let write_char = self.write_char.as_ref().ok_or(AppError::NotConnected)?;

        // Check if still connected
        if !peripheral.is_connected().await.unwrap_or(false) {
            return Err(AppError::Ble("BLE: Not connected".into()));
        }

        self.drain_notifications().await;

        log::debug!("BLE TX: {:02X?}", packet);
        peripheral.write(write_char, packet, WriteType::WithResponse).await
            .map_err(|e| AppError::Ble(format!("BLE write: {e}")))?;

        // Wait for our response (with retry — skip notifications that don't match our command)
        let cmd_bytes = &packet[4..7]; // CMD0, CMD1, CMD2 from our packet
        for _ in 0..5 {
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                self.response_notify.notified(),
            ).await;

            if result.is_err() {
                return Err(AppError::Ble("BLE response timeout".into()));
            }

            let resp = self.response.lock().await;
            if let Some(ref data) = *resp {
                // Check if response matches our command (bytes 4-6 should match)
                if data.len() >= 7 && &data[4..7] == cmd_bytes {
                    return Ok(data.clone());
                }
                // Not our response — could be an async notification from a prior SET
                log::debug!("BLE: Skipping non-matching response: {:02X?}", data);
            }
            drop(resp);
        }

        Err(AppError::Ble("BLE: no matching response after retries".into()))
    }

    fn parse_single_byte_response(&self, raw: &[u8]) -> Result<u8> {
        let (_cmd, data) = parse_response(raw)
            .ok_or_else(|| AppError::Ble("Invalid response".into()))?;
        data.first().copied()
            .ok_or_else(|| AppError::Ble("Empty response data".into()))
    }

    fn parse_two_byte_gain(&self, raw: &[u8]) -> Result<f64> {
        let (_cmd, data) = parse_response(raw)
            .ok_or_else(|| AppError::Ble("Invalid response".into()))?;
        if data.len() < 2 {
            return Err(AppError::Ble("Response too short for gain value".into()));
        }
        Ok(ble_protocol::parse_ble_gain(data[0], data[1]))
    }

    fn parse_multi_byte_response(&self, raw: &[u8]) -> Result<Vec<u8>> {
        let (_cmd, data) = parse_response(raw)
            .ok_or_else(|| AppError::Ble("Invalid response".into()))?;
        Ok(data)
    }

    async fn send_set(&self, packet: &[u8]) -> Result<()> {
        let peripheral = self.peripheral.as_ref().ok_or(AppError::NotConnected)?;
        let write_char = self.write_char.as_ref().ok_or(AppError::NotConnected)?;

        self.drain_notifications().await;

        log::debug!("BLE TX (set): {:02X?}", packet);
        peripheral.write(write_char, packet, WriteType::WithResponse).await
            .map_err(|e| AppError::Ble(format!("BLE write: {e}")))?;

        // The K13 sends 1-3 notifications after a SET (ACK + status updates).
        // Wait for the first ACK, then drain any trailing notifications.
        let _ = tokio::time::timeout(
            Duration::from_secs(3),
            self.response_notify.notified(),
        ).await;

        // Give the device a moment, then drain extra notifications
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.drain_notifications().await;

        Ok(())
    }

    // ---- Input Source ----

    pub async fn get_input_source(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_input_source()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_input_source(&self, source: u8) -> Result<()> {
        self.send_set(&set_input_source(source)).await
    }

    // ---- Light Switch ----

    pub async fn get_light_switch(&self, zone: u8) -> Result<bool> {
        let resp = self.send_and_receive(&get_light_switch(zone)).await?;
        Ok(self.parse_single_byte_response(&resp)? != 0)
    }

    pub async fn set_light_switch(&self, zone: u8, on: bool) -> Result<()> {
        self.send_set(&set_light_switch(zone, on)).await
    }

    // ---- Light Mode ----

    pub async fn get_light_mode(&self, zone: u8) -> Result<u8> {
        let resp = self.send_and_receive(&get_light_mode(zone)).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_light_mode(&self, zone: u8, mode: u8) -> Result<()> {
        self.send_set(&set_light_mode(zone, mode)).await
    }

    // ---- Light Color ----

    pub async fn get_light_color(&self, zone: u8) -> Result<u8> {
        let resp = self.send_and_receive(&get_light_color(zone)).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_light_color(&self, zone: u8, color: u8) -> Result<()> {
        self.send_set(&set_light_color(zone, color)).await
    }

    // ---- Volume ----

    pub async fn get_volume(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_volume()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_volume(&self, vol: u8) -> Result<()> {
        self.send_set(&set_volume(vol)).await
    }

    // ---- Gain Mode ----

    pub async fn get_gain_mode(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_gain_mode()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_gain_mode(&self, mode: u8) -> Result<()> {
        self.send_set(&set_gain_mode(mode)).await
    }

    // ---- Channel Balance ----

    pub async fn get_channel_balance(&self) -> Result<i8> {
        let resp = self.send_and_receive(&get_channel_balance()).await?;
        let data = self.parse_multi_byte_response(&resp)?;
        if data.len() < 2 {
            return Err(AppError::Ble("Response too short for channel balance".into()));
        }
        let direction = data[0];
        let magnitude = data[1];
        if direction == 0x01 {
            Ok(magnitude as i8)
        } else {
            Ok(-(magnitude as i8))
        }
    }

    pub async fn set_channel_balance(&self, balance: i8) -> Result<()> {
        let (direction, magnitude) = if balance >= 0 {
            (0x01u8, balance as u8)
        } else {
            (0x00u8, (-balance) as u8)
        };
        self.send_set(&set_channel_balance(direction, magnitude)).await
    }

    // ---- SPDIF Out ----

    pub async fn get_spdif_out(&self) -> Result<bool> {
        let resp = self.send_and_receive(&get_spdif_out()).await?;
        Ok(self.parse_single_byte_response(&resp)? != 0)
    }

    pub async fn set_spdif_out(&self, enabled: bool) -> Result<()> {
        self.send_set(&set_spdif_out(enabled)).await
    }

    // ---- DAC Filter ----

    pub async fn get_dac_filter(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_dac_filter()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_dac_filter(&self, filter: u8) -> Result<()> {
        self.send_set(&set_dac_filter(filter)).await
    }

    // ---- Harmonic Mode ----

    pub async fn get_harmonic_mode(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_harmonic_mode()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_harmonic_mode(&self, mode: u8) -> Result<()> {
        self.send_set(&set_harmonic_mode(mode)).await
    }

    // ---- Auto Power Off ----

    pub async fn get_auto_power_off(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_auto_power_off()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_auto_power_off(&self, value: u8) -> Result<()> {
        self.send_set(&set_auto_power_off(value)).await
    }

    // ---- EQ Switch ----

    pub async fn get_eq_switch(&self) -> Result<bool> {
        let resp = self.send_and_receive(&get_eq_switch()).await?;
        Ok(self.parse_single_byte_response(&resp)? != 0)
    }

    pub async fn set_eq_switch(&self, enabled: bool) -> Result<()> {
        self.send_set(&set_eq_switch(enabled)).await
    }

    // ---- EQ Preset ----

    pub async fn get_eq_preset(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_eq_preset()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_eq_preset(&self, preset: u8) -> Result<()> {
        self.send_set(&set_eq_preset(preset)).await
    }

    // ---- EQ Preamp ----

    pub async fn get_eq_preamp(&self) -> Result<f64> {
        let resp = self.send_and_receive(&get_eq_preamp()).await?;
        self.parse_two_byte_gain(&resp)
    }

    pub async fn set_eq_preamp(&self, gain: f64) -> Result<()> {
        self.send_set(&set_eq_preamp(gain)).await
    }

    // ---- EQ Bands ----

    pub async fn get_eq_bands(&self) -> Result<Vec<ble_protocol::BleEqBand>> {
        let resp = self.send_and_receive(&get_eq_bands_batch(0, 9)).await?;
        let data = self.parse_multi_byte_response(&resp)?;
        Ok(ble_protocol::parse_eq_band_response(&data))
    }

    pub async fn set_eq_band(&self, index: u8, freq: u16, gain: f64, q: f64, filter_type: u8) -> Result<()> {
        self.send_set(&set_eq_band_single(index, freq, gain, q, filter_type)).await
    }

    pub async fn set_all_eq_bands(&self, bands: &[ble_protocol::BleEqBand]) -> Result<()> {
        self.send_set(&set_eq_bands_batch(bands)).await
    }

    // ---- Display Mode ----

    pub async fn get_display_mode(&self) -> Result<bool> {
        let resp = self.send_and_receive(&get_display_mode()).await?;
        Ok(self.parse_single_byte_response(&resp)? != 0)
    }

    pub async fn set_display_mode(&self, on: bool) -> Result<()> {
        self.send_set(&set_display_mode(on)).await
    }

    // ---- Screen Brightness ----

    pub async fn get_screen_brightness(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_screen_brightness()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn set_screen_brightness(&self, level: u8) -> Result<()> {
        self.send_set(&set_screen_brightness(level)).await
    }

    // ---- Info (read-only) ----

    pub async fn get_firmware_version(&self) -> Result<String> {
        let resp = self.send_and_receive(&get_firmware_version()).await?;
        let data = self.parse_multi_byte_response(&resp)?;
        Ok(String::from_utf8_lossy(&data).to_string())
    }

    pub async fn get_bt_codec(&self) -> Result<u8> {
        let resp = self.send_and_receive(&get_bt_codec()).await?;
        self.parse_single_byte_response(&resp)
    }

    pub async fn get_sample_rate_info(&self) -> Result<(bool, u8, u16)> {
        let resp = self.send_and_receive(&get_sample_rate_info()).await?;
        let data = self.parse_multi_byte_response(&resp)?;
        if data.len() < 4 {
            return Err(AppError::Ble("Response too short for sample rate info".into()));
        }
        let is_dsd = data[0] != 0;
        let bit_depth = data[1];
        let sample_rate = u16::from_be_bytes([data[2], data[3]]);
        Ok((is_dsd, bit_depth, sample_rate))
    }
}
