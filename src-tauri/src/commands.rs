use tauri::State;
use crate::device::{EqBand, SharedDevice};
use crate::autoeq::{AutoEqHeadphone, AutoEqProfile};

#[tauri::command]
pub fn connect_device(device: State<SharedDevice>) -> Result<String, String> {
    let mut dev = device.lock();
    dev.connect()
}

#[tauri::command]
pub fn disconnect_device(device: State<SharedDevice>) -> Result<(), String> {
    let mut dev = device.lock();
    dev.disconnect();
    Ok(())
}

#[tauri::command]
pub fn is_connected(device: State<SharedDevice>) -> bool {
    device.lock().is_connected()
}

#[tauri::command]
pub fn get_device_name(device: State<SharedDevice>) -> Option<String> {
    device.lock().product_name().map(|s| s.to_string())
}

// ---- EQ Commands ----

#[tauri::command]
pub fn get_eq_count(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_eq_count()
}

#[tauri::command]
pub fn get_eq_band(device: State<SharedDevice>, index: u8) -> Result<EqBand, String> {
    device.lock().get_eq_band(index)
}

#[tauri::command]
pub fn get_all_eq_bands(device: State<SharedDevice>) -> Result<Vec<EqBand>, String> {
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
) -> Result<(), String> {
    device.lock().set_eq_band(index, frequency, gain, q_value, filter_type)
}

#[tauri::command]
pub fn get_eq_preset(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_eq_preset()
}

#[tauri::command]
pub fn set_eq_preset(device: State<SharedDevice>, preset: u8) -> Result<(), String> {
    device.lock().set_eq_preset(preset)
}

#[tauri::command]
pub fn get_eq_global_gain(device: State<SharedDevice>) -> Result<f64, String> {
    device.lock().get_eq_global_gain()
}

#[tauri::command]
pub fn set_eq_global_gain(device: State<SharedDevice>, gain: f64) -> Result<(), String> {
    device.lock().set_eq_global_gain(gain)
}

#[tauri::command]
pub fn get_eq_switch(device: State<SharedDevice>) -> Result<bool, String> {
    device.lock().get_eq_switch()
}

#[tauri::command]
pub fn set_eq_switch(device: State<SharedDevice>, enabled: bool) -> Result<(), String> {
    device.lock().set_eq_switch(enabled)
}

#[tauri::command]
pub fn save_eq(device: State<SharedDevice>, preset: u8) -> Result<(), String> {
    device.lock().save_eq(preset)
}

#[tauri::command]
pub fn reset_eq(device: State<SharedDevice>) -> Result<(), String> {
    device.lock().reset_eq()
}

// ---- Config Commands ----

#[tauri::command]
pub fn get_firmware_version(device: State<SharedDevice>) -> Result<String, String> {
    device.lock().get_firmware_version()
}

#[tauri::command]
pub fn get_vol_max(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_vol_max()
}

#[tauri::command]
pub fn set_vol_max(device: State<SharedDevice>, val: u8) -> Result<(), String> {
    device.lock().set_vol_max(val)
}

#[tauri::command]
pub fn get_vol_output(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_vol_output()
}

#[tauri::command]
pub fn set_vol_output(device: State<SharedDevice>, val: u8) -> Result<(), String> {
    device.lock().set_vol_output(val)
}

#[tauri::command]
pub fn get_vol_output_switch(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_vol_output_switch()
}

#[tauri::command]
pub fn set_vol_output_switch(device: State<SharedDevice>, val: u8) -> Result<(), String> {
    device.lock().set_vol_output_switch(val)
}

#[tauri::command]
pub fn get_mic_switch(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_mic_switch()
}

#[tauri::command]
pub fn set_mic_switch(device: State<SharedDevice>, val: u8) -> Result<(), String> {
    device.lock().set_mic_switch(val)
}

#[tauri::command]
pub fn get_mic_monitor_vol(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_mic_monitor_vol()
}

#[tauri::command]
pub fn set_mic_monitor_vol(device: State<SharedDevice>, val: u8) -> Result<(), String> {
    device.lock().set_mic_monitor_vol(val)
}

#[tauri::command]
pub fn get_screen_orientation(device: State<SharedDevice>) -> Result<u8, String> {
    device.lock().get_screen_orientation()
}

#[tauri::command]
pub fn set_screen_orientation(device: State<SharedDevice>, val: u8) -> Result<(), String> {
    device.lock().set_screen_orientation(val)
}

#[tauri::command]
pub fn get_channel_balance(device: State<SharedDevice>) -> Result<i8, String> {
    device.lock().get_channel_balance()
}

#[tauri::command]
pub fn set_channel_balance(device: State<SharedDevice>, val: i8) -> Result<(), String> {
    device.lock().set_channel_balance(val)
}

#[tauri::command]
pub fn get_preset_name(device: State<SharedDevice>, index: u8) -> Result<String, String> {
    device.lock().get_preset_name(index)
}

#[tauri::command]
pub fn set_preset_name(device: State<SharedDevice>, index: u8, name: String) -> Result<(), String> {
    device.lock().set_preset_name(index, &name)
}

// ---- AutoEQ Commands ----

#[tauri::command]
pub fn fetch_autoeq_index() -> Result<Vec<AutoEqHeadphone>, String> {
    crate::autoeq::fetch_index()
}

#[tauri::command]
pub fn fetch_autoeq_profile(path: String) -> Result<AutoEqProfile, String> {
    crate::autoeq::fetch_parametric_eq(&path)
}
