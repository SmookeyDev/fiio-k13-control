/// FiiO K13 R2R HID Protocol
/// Reverse-engineered from fiiocontrol.fiio.com
///
/// Packet structure: [HEAD, START, 0, 0, CMD, DATA_LEN, ...DATA, 0, STOP(0xEE)]
/// GET: HEAD=0xBB, START=0x0B
/// SET: HEAD=0xAA, START=0x0A

const GET_HEAD: u8 = 0xBB;
const GET_START: u8 = 0x0B;
const SET_HEAD: u8 = 0xAA;
const SET_START: u8 = 0x0A;
const STOP: u8 = 0xEE;

// Command IDs (active)
const CMD_EQ_BAND_ITEM: u8 = 0x15;
const CMD_EQ_PRESET: u8 = 0x16;
const CMD_EQ_GLOBAL_GAIN: u8 = 0x17;
const CMD_EQ_COUNT: u8 = 0x18;
const CMD_EQ_SAVE: u8 = 0x19;
const CMD_EQ_SWITCH: u8 = 0x1A;
const CMD_EQ_RESET: u8 = 0x1B;
const CMD_PRESET_NAME: u8 = 0x30;

pub const FIIO_VENDOR_ID: u16 = 10610;

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

// ---- Codec helpers ----

pub fn parse_gain(b1: u8, b2: u8) -> f64 {
    let raw = ((b1 as u16) << 8) | (b2 as u16);
    if raw & 0x8000 != 0 {
        let val = ((raw ^ 0xFFFF) + 1) as f64;
        -val / 10.0
    } else {
        raw as f64 / 10.0
    }
}

pub fn encode_gain(gain: f64) -> [u8; 2] {
    let val = (gain * 10.0).round() as i16;
    val.to_be_bytes()
}

pub fn parse_q_value(b1: u8, b2: u8) -> f64 {
    let raw = ((b1 as u16) << 8) | (b2 as u16);
    raw as f64 / 100.0
}

pub fn parse_frequency(b1: u8, b2: u8) -> u16 {
    ((b1 as u16) << 8) | (b2 as u16)
}

// ---- EQ packets ----

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

// ---- Preset name packets ----

pub fn get_preset_name(index: u8) -> Vec<u8> {
    build_get_packet(CMD_PRESET_NAME, &[index])
}

pub fn set_preset_name(index: u8, name: &str) -> Vec<u8> {
    let name_bytes: Vec<u8> = name.bytes().take(8).collect();
    let mut data = vec![index];
    data.extend_from_slice(&name_bytes);
    build_set_packet(CMD_PRESET_NAME, &data)
}
