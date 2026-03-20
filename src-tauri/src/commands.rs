use tauri::State;
use crate::device::{EqBand, SharedDevice};
use crate::ble_device::SharedBleDevice;
use crate::error::AppError;
use crate::autoeq::{AutoEqHeadphone, AutoEqProfile};

type Result<T> = std::result::Result<T, AppError>;

// ---- USB Connection ----

#[tauri::command]
pub fn connect_device(device: State<SharedDevice>) -> Result<String> {
    device.lock().connect()
}

#[tauri::command]
pub fn disconnect_device(device: State<SharedDevice>) {
    device.lock().disconnect();
}

#[tauri::command]
pub fn is_connected(device: State<SharedDevice>) -> bool {
    device.lock().is_connected()
}

#[tauri::command]
pub fn get_device_name(device: State<SharedDevice>) -> Option<String> {
    device.lock().product_name().map(|s| s.to_string())
}

// ---- USB EQ ----

#[tauri::command]
pub fn get_eq_count(device: State<SharedDevice>) -> Result<u8> {
    device.lock().get_eq_count()
}

#[tauri::command]
pub fn get_eq_band(device: State<SharedDevice>, index: u8) -> Result<EqBand> {
    device.lock().get_eq_band(index)
}

#[tauri::command]
pub fn get_all_eq_bands(device: State<SharedDevice>) -> Result<Vec<EqBand>> {
    device.lock().get_all_eq_bands()
}

#[tauri::command]
pub fn set_eq_band(device: State<SharedDevice>, index: u8, frequency: u16, gain: f64, q_value: f64, filter_type: u8) -> Result<()> {
    device.lock().set_eq_band(index, frequency, gain, q_value, filter_type)
}

#[tauri::command]
pub fn get_eq_preset(device: State<SharedDevice>) -> Result<u8> {
    device.lock().get_eq_preset()
}

#[tauri::command]
pub fn set_eq_preset(device: State<SharedDevice>, preset: u8) -> Result<()> {
    device.lock().set_eq_preset(preset)
}

#[tauri::command]
pub fn get_eq_global_gain(device: State<SharedDevice>) -> Result<f64> {
    device.lock().get_eq_global_gain()
}

#[tauri::command]
pub fn set_eq_global_gain(device: State<SharedDevice>, gain: f64) -> Result<()> {
    device.lock().set_eq_global_gain(gain)
}

#[tauri::command]
pub fn get_eq_switch(device: State<SharedDevice>) -> Result<bool> {
    device.lock().get_eq_switch()
}

#[tauri::command]
pub fn set_eq_switch(device: State<SharedDevice>, enabled: bool) -> Result<()> {
    device.lock().set_eq_switch(enabled)
}

#[tauri::command]
pub fn save_eq(device: State<SharedDevice>, preset: u8) -> Result<()> {
    device.lock().save_eq(preset)
}

#[tauri::command]
pub fn reset_eq(device: State<SharedDevice>) -> Result<()> {
    device.lock().reset_eq()
}

// ---- USB Preset Names ----

#[tauri::command]
pub fn get_preset_name(device: State<SharedDevice>, index: u8) -> Result<String> {
    device.lock().get_preset_name(index)
}

#[tauri::command]
pub fn set_preset_name(device: State<SharedDevice>, index: u8, name: String) -> Result<()> {
    device.lock().set_preset_name(index, &name)
}

// ---- BLE Connection ----

#[tauri::command]
pub async fn ble_connect(device: State<'_, SharedBleDevice>) -> Result<String> {
    device.lock().await.connect().await
}

#[tauri::command]
pub async fn ble_disconnect(device: State<'_, SharedBleDevice>) -> Result<()> {
    device.lock().await.disconnect().await;
    Ok(())
}

#[tauri::command]
pub async fn ble_is_connected(device: State<'_, SharedBleDevice>) -> Result<bool> {
    Ok(device.lock().await.check_connected().await)
}

#[tauri::command]
pub async fn ble_get_device_name(device: State<'_, SharedBleDevice>) -> Result<Option<String>> {
    Ok(device.lock().await.device_name().map(|s| s.to_string()))
}

// ---- BLE Input Source ----

#[tauri::command]
pub async fn ble_get_input_source(device: State<'_, SharedBleDevice>) -> Result<u8> {
    device.lock().await.get_input_source().await
}

#[tauri::command]
pub async fn ble_set_input_source(device: State<'_, SharedBleDevice>, source: u8) -> Result<()> {
    device.lock().await.set_input_source(source).await
}

// ---- BLE Indicator Lights ----

#[tauri::command]
pub async fn ble_get_light_switch(device: State<'_, SharedBleDevice>, zone: u8) -> Result<bool> {
    device.lock().await.get_light_switch(zone).await
}

#[tauri::command]
pub async fn ble_set_light_switch(device: State<'_, SharedBleDevice>, zone: u8, on: bool) -> Result<()> {
    device.lock().await.set_light_switch(zone, on).await
}

#[tauri::command]
pub async fn ble_get_light_mode(device: State<'_, SharedBleDevice>, zone: u8) -> Result<u8> {
    device.lock().await.get_light_mode(zone).await
}

#[tauri::command]
pub async fn ble_set_light_mode(device: State<'_, SharedBleDevice>, zone: u8, mode: u8) -> Result<()> {
    device.lock().await.set_light_mode(zone, mode).await
}

#[tauri::command]
pub async fn ble_get_light_color(device: State<'_, SharedBleDevice>, zone: u8) -> Result<u8> {
    device.lock().await.get_light_color(zone).await
}

#[tauri::command]
pub async fn ble_set_light_color(device: State<'_, SharedBleDevice>, zone: u8, color: u8) -> Result<()> {
    device.lock().await.set_light_color(zone, color).await
}

// ---- BLE Info ----

#[tauri::command]
pub async fn ble_get_firmware_version(device: State<'_, SharedBleDevice>) -> Result<String> {
    device.lock().await.get_firmware_version().await
}

// ---- AutoEQ ----

#[tauri::command]
pub fn fetch_autoeq_index() -> std::result::Result<Vec<AutoEqHeadphone>, AppError> {
    crate::autoeq::fetch_index()
}

#[tauri::command]
pub fn fetch_autoeq_profile(path: String) -> std::result::Result<AutoEqProfile, AppError> {
    crate::autoeq::fetch_parametric_eq(&path)
}
