/// FiiO K13 R2R HID Protocol
/// Reverse-engineered from fiiocontrol.fiio.com
///
/// Packet structure: [HEAD, START, 0, 0, CMD, DATA_LEN, ...DATA, 0, STOP(0xEE)]
/// GET: HEAD=0xBB(187), START=0x0B(11)
/// SET: HEAD=0xAA(170), START=0x0A(10)

const GET_HEAD: u8 = 0xBB;
const GET_START: u8 = 0x0B;
const SET_HEAD: u8 = 0xAA;
const SET_START: u8 = 0x0A;
const STOP: u8 = 0xEE;

// Command IDs
pub const CMD_VOL_MAX: u8 = 0x01;
pub const CMD_VOL_OUTPUT: u8 = 0x02;
pub const CMD_CHANNEL_BALANCE: u8 = 0x07;
pub const CMD_VOL_OUTPUT_SWITCH: u8 = 0x08;
pub const CMD_FIRMWARE_VERSION: u8 = 0x0B;
pub const CMD_SCREEN_ORIENTATION: u8 = 0x10;
pub const CMD_MIC_SWITCH: u8 = 0x12;
pub const CMD_EQ_BAND_ITEM: u8 = 0x15;
pub const CMD_EQ_PRESET: u8 = 0x16;
pub const CMD_EQ_GLOBAL_GAIN: u8 = 0x17;
pub const CMD_EQ_COUNT: u8 = 0x18;
pub const CMD_EQ_SAVE: u8 = 0x19;
pub const CMD_EQ_SWITCH: u8 = 0x1A;
pub const CMD_EQ_RESET: u8 = 0x1B;
pub const CMD_MIC_MONITOR_VOL: u8 = 0x1E;
pub const CMD_GAIN_MODE: u8 = 0x1D;
pub const CMD_PRESET_NAME: u8 = 0x30;

// K13 R2R vendor ID
pub const FIIO_VENDOR_ID: u16 = 10610;
pub const REPORT_ID: u8 = 7;

fn build_get_packet(cmd: u8, data: &[u8]) -> Vec<u8> {
    let len = data.len() as u8;
    let mut pkt = vec![GET_HEAD, GET_START, 0, 0, cmd, len];
    pkt.extend_from_slice(data);
    pkt.push(0);
    pkt.push(STOP);
    pkt
}

fn build_set_packet(cmd: u8, data: &[u8]) -> Vec<u8> {
    let len = data.len() as u8;
    let mut pkt = vec![SET_HEAD, SET_START, 0, 0, cmd, len];
    pkt.extend_from_slice(data);
    pkt.push(0);
    pkt.push(STOP);
    pkt
}

/// Parse gain from two bytes (signed, x10)
pub fn parse_gain(b1: u8, b2: u8) -> f64 {
    let raw = ((b1 as u16) << 8) | (b2 as u16);
    if raw & 0x8000 != 0 {
        let val = ((raw ^ 0xFFFF) + 1) as f64;
        -val / 10.0
    } else {
        raw as f64 / 10.0
    }
}

/// Encode gain to two bytes
pub fn encode_gain(gain: f64) -> [u8; 2] {
    let mut buf = [0u8; 2];
    let val = (gain * 10.0).round() as i16;
    let bytes = val.to_be_bytes();
    // The protocol swaps byte order for gain encoding
    buf[0] = bytes[0];
    buf[1] = bytes[1];
    buf
}

/// Parse Q value from two bytes (x100)
pub fn parse_q_value(b1: u8, b2: u8) -> f64 {
    let raw = ((b1 as u16) << 8) | (b2 as u16);
    raw as f64 / 100.0
}

/// Parse frequency from two bytes
pub fn parse_frequency(b1: u8, b2: u8) -> u16 {
    ((b1 as u16) << 8) | (b2 as u16)
}

/// Extract payload from response: data starts at byte 8, length at byte 3
pub fn extract_payload(response: &[u8]) -> &[u8] {
    if response.len() < 8 {
        return &[];
    }
    let len = response[3] as usize;
    let end = (8 + len).min(response.len());
    &response[8..end]
}

// ---- GET commands ----

pub fn get_eq_count() -> Vec<u8> {
    build_get_packet(CMD_EQ_COUNT, &[])
}

pub fn get_eq_band_item(index: u8) -> Vec<u8> {
    build_get_packet(CMD_EQ_BAND_ITEM, &[index])
}

pub fn get_eq_preset() -> Vec<u8> {
    build_get_packet(CMD_EQ_PRESET, &[])
}

pub fn get_eq_global_gain() -> Vec<u8> {
    build_get_packet(CMD_EQ_GLOBAL_GAIN, &[])
}

pub fn get_eq_switch() -> Vec<u8> {
    build_get_packet(CMD_EQ_SWITCH, &[])
}

pub fn get_firmware_version() -> Vec<u8> {
    build_get_packet(CMD_FIRMWARE_VERSION, &[])
}

pub fn get_vol_max() -> Vec<u8> {
    build_get_packet(CMD_VOL_MAX, &[])
}

pub fn get_vol_output() -> Vec<u8> {
    build_get_packet(CMD_VOL_OUTPUT, &[])
}

pub fn get_vol_output_switch() -> Vec<u8> {
    build_get_packet(CMD_VOL_OUTPUT_SWITCH, &[])
}

pub fn get_mic_switch() -> Vec<u8> {
    build_get_packet(CMD_MIC_SWITCH, &[])
}

pub fn get_mic_monitor_vol() -> Vec<u8> {
    build_get_packet(CMD_MIC_MONITOR_VOL, &[])
}

pub fn get_screen_orientation() -> Vec<u8> {
    build_get_packet(CMD_SCREEN_ORIENTATION, &[])
}

pub fn get_channel_balance() -> Vec<u8> {
    build_get_packet(CMD_CHANNEL_BALANCE, &[])
}

pub fn get_gain_mode() -> Vec<u8> {
    build_get_packet(CMD_GAIN_MODE, &[])
}

pub fn get_preset_name(index: u8) -> Vec<u8> {
    build_get_packet(CMD_PRESET_NAME, &[index])
}

// ---- SET commands ----

pub fn set_eq_band_item(index: u8, freq: u16, gain: f64, q: f64, filter_type: u8) -> Vec<u8> {
    let g = encode_gain(gain);
    let freq_hi = ((freq & 0xFF00) >> 8) as u8;
    let freq_lo = (freq & 0x00FF) as u8;
    let q_raw = (q * 100.0) as u16;
    let q_hi = ((q_raw & 0xFF00) >> 8) as u8;
    let q_lo = (q_raw & 0x00FF) as u8;
    build_set_packet(CMD_EQ_BAND_ITEM, &[index, g[0], g[1], freq_hi, freq_lo, q_hi, q_lo, filter_type])
}

pub fn set_eq_preset(preset: u8) -> Vec<u8> {
    build_set_packet(CMD_EQ_PRESET, &[preset])
}

pub fn set_eq_global_gain(gain: f64) -> Vec<u8> {
    let g = encode_gain(gain);
    build_set_packet(CMD_EQ_GLOBAL_GAIN, &[g[0], g[1]])
}

pub fn set_eq_save(preset: u8) -> Vec<u8> {
    build_set_packet(CMD_EQ_SAVE, &[preset])
}

pub fn set_eq_switch(enabled: u8) -> Vec<u8> {
    build_set_packet(CMD_EQ_SWITCH, &[enabled])
}

pub fn set_eq_reset() -> Vec<u8> {
    build_set_packet(CMD_EQ_RESET, &[])
}

pub fn set_vol_max(val: u8) -> Vec<u8> {
    build_set_packet(CMD_VOL_MAX, &[val])
}

pub fn set_vol_output(val: u8) -> Vec<u8> {
    build_set_packet(CMD_VOL_OUTPUT, &[val])
}

pub fn set_vol_output_switch(val: u8) -> Vec<u8> {
    build_set_packet(CMD_VOL_OUTPUT_SWITCH, &[val])
}

pub fn set_mic_switch(val: u8) -> Vec<u8> {
    build_set_packet(CMD_MIC_SWITCH, &[val])
}

pub fn set_mic_monitor_vol(val: u8) -> Vec<u8> {
    build_set_packet(CMD_MIC_MONITOR_VOL, &[val])
}

pub fn set_screen_orientation(val: u8) -> Vec<u8> {
    build_set_packet(CMD_SCREEN_ORIENTATION, &[val])
}

pub fn set_channel_balance(val: i8) -> Vec<u8> {
    let data = if val > 0 {
        vec![0, val as u8]
    } else if val < 0 {
        vec![val as u8, 0]
    } else {
        vec![0, 0]
    };
    build_set_packet(CMD_CHANNEL_BALANCE, &data)
}

pub fn set_preset_name(index: u8, name: &str) -> Vec<u8> {
    let name_bytes: Vec<u8> = name.bytes().take(8).collect();
    let mut data = vec![index];
    data.extend_from_slice(&name_bytes);
    build_set_packet(CMD_PRESET_NAME, &data)
}
