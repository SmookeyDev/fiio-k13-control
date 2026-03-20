use serde::Serialize;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Device not connected")]
    NotConnected,

    #[error("No FiiO K13 R2R found. Make sure it's connected via USB.")]
    DeviceNotFound,

    #[error("USB: {0}")]
    Usb(#[from] rusb::Error),

    #[error("BLE: {0}")]
    Ble(String),

    #[error("Network: {0}")]
    Network(#[from] reqwest::Error),

    #[error("{0}")]
    Protocol(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
