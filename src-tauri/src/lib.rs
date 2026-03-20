mod autoeq;
mod ble_device;
mod ble_protocol;
mod commands;
mod device;
mod error;
mod protocol;

use device::create_shared_device;
use ble_device::create_shared_ble_device;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .init();

    // Fix WebKitGTK crash on Wayland (Fedora/GNOME)
    // SAFETY: called before any threads are spawned
    unsafe {
        if std::env::var("GDK_BACKEND").is_err() {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(create_shared_device())
        .manage(create_shared_ble_device())
        .invoke_handler(tauri::generate_handler![
            // USB Connection
            commands::connect_device,
            commands::disconnect_device,
            commands::is_connected,
            commands::get_device_name,
            // EQ (USB)
            commands::get_eq_count,
            commands::get_eq_band,
            commands::get_all_eq_bands,
            commands::set_eq_band,
            commands::get_eq_preset,
            commands::set_eq_preset,
            commands::get_eq_global_gain,
            commands::set_eq_global_gain,
            commands::get_eq_switch,
            commands::set_eq_switch,
            commands::save_eq,
            commands::reset_eq,
            // Preset names (USB)
            commands::get_preset_name,
            commands::set_preset_name,
            // BLE Connection
            commands::ble_connect,
            commands::ble_disconnect,
            commands::ble_is_connected,
            commands::ble_get_device_name,
            // BLE Settings
            commands::ble_get_input_source,
            commands::ble_set_input_source,
            commands::ble_get_light_switch,
            commands::ble_set_light_switch,
            commands::ble_get_light_mode,
            commands::ble_set_light_mode,
            commands::ble_get_light_color,
            commands::ble_set_light_color,
            // BLE Info
            commands::ble_get_firmware_version,
            // AutoEQ
            commands::fetch_autoeq_index,
            commands::fetch_autoeq_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
