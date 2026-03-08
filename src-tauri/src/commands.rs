use tauri::State;
use crate::device::{EqBand, SharedDevice};
use crate::error::AppError;
use crate::autoeq::{AutoEqHeadphone, AutoEqProfile};

type Result<T> = std::result::Result<T, AppError>;

// ---- Connection ----

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

// ---- EQ ----

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
pub fn set_eq_band(
    device: State<SharedDevice>,
    index: u8,
    frequency: u16,
    gain: f64,
    q_value: f64,
    filter_type: u8,
) -> Result<()> {
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

// ---- Preset Names ----

#[tauri::command]
pub fn get_preset_name(device: State<SharedDevice>, index: u8) -> Result<String> {
    device.lock().get_preset_name(index)
}

#[tauri::command]
pub fn set_preset_name(device: State<SharedDevice>, index: u8, name: String) -> Result<()> {
    device.lock().set_preset_name(index, &name)
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
