#![allow(dead_code)]
//! FiiO K13 R2R BLE Protocol
//!
//! Reverse-engineered from FiiO Control Android APK (device type 43).
//!
//! Packet structure: `[F1, 10, 00, LEN, CMD0, CMD1, CMD2, DATA..., FF]`
//! - GET commands: CMD0 high nibble = `0x0X`
//! - SET commands: CMD0 high nibble = `0x1X` (i.e. `CMD0 | 0x10`)

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Packet framing
// ---------------------------------------------------------------------------

const HEADER: [u8; 3] = [0xF1, 0x10, 0x00];
const FOOTER: u8 = 0xFF;

// ---------------------------------------------------------------------------
// BLE UUIDs
// ---------------------------------------------------------------------------

pub const SERVICE_UUID: &str = "00001100-04a5-1000-1000-40ed981a04a5";
pub const WRITE_UUID: &str = "00001101-04a5-1000-1000-40ed981a04a5";
pub const NOTIFY_UUID: &str = "00001102-04a5-1000-1000-40ed981a04a5";

// ---------------------------------------------------------------------------
// Input source constants
// ---------------------------------------------------------------------------

pub const INPUT_USB: u8 = 0x01;
pub const INPUT_COAXIAL: u8 = 0x04;
pub const INPUT_OPTICAL: u8 = 0x08;
pub const INPUT_BLUETOOTH: u8 = 0x20;

// ---------------------------------------------------------------------------
// Light constants
// ---------------------------------------------------------------------------

/// Light color values
pub const COLOR_FOLLOW_AUDIO: u8 = 0x00;
pub const COLOR_RED: u8 = 0x01;
pub const COLOR_BLUE: u8 = 0x02;
pub const COLOR_TURQUOISE: u8 = 0x03;
pub const COLOR_PURPLE: u8 = 0x04;
pub const COLOR_YELLOW: u8 = 0x05;
pub const COLOR_WHITE: u8 = 0x06;
pub const COLOR_GREEN: u8 = 0x07;
pub const COLOR_CYCLE: u8 = 0x08;

/// Light mode values
pub const MODE_ALWAYS_ON: u8 = 0x00;
pub const MODE_BREATHE: u8 = 0x01;

/// Light zones
pub const ZONE_TOP: u8 = 0x02;
pub const ZONE_KNOB: u8 = 0x03;

// ---------------------------------------------------------------------------
// Gain mode constants
// ---------------------------------------------------------------------------

pub const GAIN_LOW: u8 = 0x00;
pub const GAIN_HIGH: u8 = 0x01;

// ---------------------------------------------------------------------------
// Channel balance direction
// ---------------------------------------------------------------------------

pub const BALANCE_LEFT: u8 = 0x00;
pub const BALANCE_RIGHT: u8 = 0x01;

// ---------------------------------------------------------------------------
// DAC filter name lookup
// ---------------------------------------------------------------------------

pub const DAC_FILTER_NAMES: &[&str] = &[
    "Sharp Roll-Off",
    "Slow Roll-Off",
    "Short Delay Sharp",
    "Short Delay Slow",
    "Super Slow",
];

// ---------------------------------------------------------------------------
// EQ filter type constants
// ---------------------------------------------------------------------------

pub const FILTER_PEAK: u8 = 0;
pub const FILTER_LOW_SHELF: u8 = 1;
pub const FILTER_HIGH_SHELF: u8 = 2;
pub const FILTER_BAND_PASS: u8 = 3;
pub const FILTER_LOW_PASS: u8 = 4;
pub const FILTER_HIGH_PASS: u8 = 5;
pub const FILTER_ALL_PASS: u8 = 6;

pub const FILTER_TYPE_NAMES: &[&str] = &[
    "Peak",
    "Low Shelf",
    "High Shelf",
    "Band Pass",
    "Low Pass",
    "High Pass",
    "All Pass",
];

// ---------------------------------------------------------------------------
// EQ preset name lookup
// ---------------------------------------------------------------------------

pub const EQ_PRESET_NAMES: &[&str] = &[
    "Custom 1",
    "Custom 2",
    "Custom 3",
    "Jazz",
    "Pop",
    "Rock",
    "Dance",
    "R&B",
    "Classical",
    "Hip-Hop",
];

// ---------------------------------------------------------------------------
// Bluetooth codec name lookup
// ---------------------------------------------------------------------------

pub const BT_CODEC_NAMES: &[&str] = &[
    "SBC",
    "AAC",
    "aptX",
    "aptX HD",
    "aptX Adaptive",
    "LDAC",
    "LHDC",
];

// ---------------------------------------------------------------------------
// GET command triplets (CMD0, CMD1, CMD2)
// ---------------------------------------------------------------------------

// Device settings -- category 0x02
pub const CMD_GET_VOLUME: [u8; 3] = [0x02, 0x01, 0x01];
pub const CMD_GET_GAIN_MODE: [u8; 3] = [0x02, 0x02, 0x01];
pub const CMD_GET_CHANNEL_BALANCE: [u8; 3] = [0x02, 0x06, 0x01];
pub const CMD_GET_SPDIF_OUT: [u8; 3] = [0x02, 0x08, 0x01];
pub const CMD_GET_DAC_FILTER: [u8; 3] = [0x02, 0x09, 0x01];
pub const CMD_GET_HARMONIC_MODE: [u8; 3] = [0x02, 0x0A, 0x01];
pub const CMD_GET_AUTO_POWER_OFF: [u8; 3] = [0x02, 0x0B, 0x01];

// EQ -- category 0x03
pub const CMD_GET_EQ_SWITCH: [u8; 3] = [0x03, 0x01, 0x01];
pub const CMD_GET_EQ_PRESET: [u8; 3] = [0x03, 0x03, 0x01];
pub const CMD_GET_EQ_PRESET_NAMES: [u8; 3] = [0x03, 0x0B, 0x01];
pub const CMD_GET_EQ_BAND_BATCH: [u8; 3] = [0x03, 0x0C, 0x01];
pub const CMD_GET_EQ_PREAMP: [u8; 3] = [0x03, 0x0D, 0x01];

// Audio info -- category 0x04
pub const CMD_GET_SAMPLE_RATE_INFO: [u8; 3] = [0x04, 0x03, 0x01];
pub const CMD_GET_DAC_CHIP_FILTER: [u8; 3] = [0x04, 0x05, 0x01];

// Display -- category 0x05
pub const CMD_GET_DISPLAY_MODE: [u8; 3] = [0x05, 0x01, 0x01];
pub const CMD_GET_SCREEN_BRIGHTNESS: [u8; 3] = [0x05, 0x05, 0x01];

// Info (read-only) -- category 0x08
pub const CMD_GET_FIRMWARE_VERSION: [u8; 3] = [0x08, 0x02, 0x01];
pub const CMD_GET_BT_CODEC: [u8; 3] = [0x08, 0x02, 0x81];

// Input source -- category 0x09
pub const CMD_GET_INPUT_SOURCE: [u8; 3] = [0x09, 0x02, 0x01];

// ---------------------------------------------------------------------------
// BLE EQ band struct
// ---------------------------------------------------------------------------

/// A single parametric EQ band for BLE communication.
///
/// Over BLE, values are transmitted as fixed-point integers:
/// - `gain`: signed i16 big-endian, value = dB * 10.0 (range -24.0 to +12.0)
/// - `frequency`: unsigned u16 big-endian, value = Hz
/// - `q_value`: unsigned u16 big-endian, value = Q * 100.0
/// - `filter_type`: u8 index (see `FILTER_*` constants)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleEqBand {
    /// Band index (0-based).
    pub index: u8,
    /// Center frequency in Hz.
    pub frequency: u16,
    /// Gain in dB (e.g. -12.0, +6.5).
    pub gain: f64,
    /// Q factor (e.g. 0.71, 1.41).
    pub q_value: f64,
    /// Filter type index (see `FILTER_*` constants).
    pub filter_type: u8,
}

// ---------------------------------------------------------------------------
// Sample rate info
// ---------------------------------------------------------------------------

/// Parsed sample rate information from CMD_GET_SAMPLE_RATE_INFO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRateInfo {
    /// True if the stream is DSD.
    pub is_dsd: bool,
    /// Bit depth (e.g. 16, 24, 32).
    pub bit_depth: u8,
    /// Sample rate in Hz (e.g. 44100, 96000).
    pub sample_rate: u32,
}

// ===========================================================================
// Packet building & parsing
// ===========================================================================

/// Build a BLE packet from a 3-byte command and optional data payload.
pub fn build_packet(cmd: [u8; 3], data: &[u8]) -> Vec<u8> {
    let payload_len = 3 + data.len();
    let total_len = payload_len + 5; // F1 10 00 LEN ... FF
    let mut pkt = Vec::with_capacity(total_len);
    pkt.extend_from_slice(&HEADER);
    pkt.push(total_len as u8);
    pkt.extend_from_slice(&cmd);
    pkt.extend_from_slice(data);
    pkt.push(FOOTER);
    pkt
}

/// Convert a GET command triplet to its SET equivalent by setting the high
/// nibble of CMD0 (`cmd[0] | 0x10`).
pub fn get_to_set(cmd: [u8; 3]) -> [u8; 3] {
    [cmd[0] | 0x10, cmd[1], cmd[2]]
}

/// Parse a BLE response packet. Returns `(cmd, data)` or `None` if the
/// packet is malformed.
pub fn parse_response(raw: &[u8]) -> Option<([u8; 3], Vec<u8>)> {
    if raw.len() < 5 {
        return None;
    }
    if raw[0] != 0xF1 || raw[1] != 0x10 {
        return None;
    }
    if raw.len() < 7 {
        return None;
    }
    let cmd = [raw[4], raw[5], raw[6]];
    let data = if raw.len() > 8 && raw[raw.len() - 1] == FOOTER {
        raw[7..raw.len() - 1].to_vec()
    } else if raw.len() > 7 {
        raw[7..].to_vec()
    } else {
        vec![]
    };
    Some((cmd, data))
}

// ---------------------------------------------------------------------------
// Response data helpers
// ---------------------------------------------------------------------------

/// Extract a single `u8` value from response data.
pub fn parse_response_u8(data: &[u8]) -> Option<u8> {
    data.first().copied()
}

/// Extract a boolean from response data (0x00 = false, anything else = true).
pub fn parse_response_bool(data: &[u8]) -> Option<bool> {
    data.first().map(|&v| v != 0)
}

/// Extract a signed i16 big-endian value from the first 2 bytes of data.
pub fn parse_response_i16_be(data: &[u8]) -> Option<i16> {
    if data.len() < 2 {
        return None;
    }
    Some(i16::from_be_bytes([data[0], data[1]]))
}

/// Extract an unsigned u16 big-endian value from the first 2 bytes of data.
pub fn parse_response_u16_be(data: &[u8]) -> Option<u16> {
    if data.len() < 2 {
        return None;
    }
    Some(u16::from_be_bytes([data[0], data[1]]))
}

/// Extract an ASCII string from response data (stops at first null or end).
pub fn parse_response_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).to_string()
}

/// Parse sample rate info from a 4-byte response `[dsd_flag, bit_depth, sr_hi, sr_lo]`.
pub fn parse_sample_rate_info(data: &[u8]) -> Option<SampleRateInfo> {
    if data.len() < 4 {
        return None;
    }
    Some(SampleRateInfo {
        is_dsd: data[0] != 0,
        bit_depth: data[1],
        sample_rate: u16::from_be_bytes([data[2], data[3]]) as u32,
    })
}

/// Parse channel balance from a 2-byte response `[direction, magnitude]`.
/// Returns a signed value: positive = right, negative = left, 0 = center.
pub fn parse_channel_balance(data: &[u8]) -> Option<i16> {
    if data.len() < 2 {
        return None;
    }
    let magnitude = data[1] as i16;
    if data[0] == BALANCE_RIGHT {
        Some(magnitude)
    } else {
        Some(-magnitude)
    }
}

/// Encode a signed balance value into `[direction, magnitude]`.
/// Positive = right, negative = left, 0 = center `[0x00, 0x00]`.
pub fn encode_channel_balance(balance: i16) -> [u8; 2] {
    if balance > 0 {
        [BALANCE_RIGHT, balance as u8]
    } else if balance < 0 {
        [BALANCE_LEFT, (-balance) as u8]
    } else {
        [0x00, 0x00]
    }
}

// ---------------------------------------------------------------------------
// EQ value encoding/decoding
// ---------------------------------------------------------------------------

/// Encode a gain in dB to 2 bytes signed i16 big-endian (dB * 10).
pub fn encode_ble_gain(db: f64) -> (u8, u8) {
    let raw = (db * 10.0).round() as i16;
    let bytes = raw.to_be_bytes();
    (bytes[0], bytes[1])
}

/// Decode a 2-byte signed i16 big-endian gain to dB.
pub fn parse_ble_gain(hi: u8, lo: u8) -> f64 {
    let raw = i16::from_be_bytes([hi, lo]);
    raw as f64 / 10.0
}

/// Encode a frequency in Hz to 2 bytes unsigned u16 big-endian.
pub fn encode_frequency(hz: u16) -> (u8, u8) {
    let bytes = hz.to_be_bytes();
    (bytes[0], bytes[1])
}

/// Decode a 2-byte unsigned u16 big-endian to frequency in Hz.
pub fn decode_frequency(hi: u8, lo: u8) -> u16 {
    u16::from_be_bytes([hi, lo])
}

/// Encode a Q factor to 2 bytes unsigned u16 big-endian (Q * 100).
pub fn encode_q(q: f64) -> (u8, u8) {
    let raw = (q * 100.0).round() as u16;
    let bytes = raw.to_be_bytes();
    (bytes[0], bytes[1])
}

/// Decode a 2-byte unsigned u16 big-endian Q value to f64.
pub fn decode_q(hi: u8, lo: u8) -> f64 {
    let raw = u16::from_be_bytes([hi, lo]);
    raw as f64 / 100.0
}

// ---------------------------------------------------------------------------
// EQ band batch encode/decode
// ---------------------------------------------------------------------------

/// Bytes per band in a batch: gain(2) + freq(2) + q(2) + filter_type(1) = 7
const BYTES_PER_BAND: usize = 7;

/// Decode a batch of EQ bands from response data.
///
/// Data layout: `[start_band, end_band, {gain(2), freq(2), q(2), filter_type(1)} * N]`
pub fn decode_eq_band_batch(data: &[u8]) -> Option<Vec<BleEqBand>> {
    if data.len() < 2 {
        return None;
    }
    let start_band = data[0];
    let end_band = data[1];
    let band_count = (end_band - start_band + 1) as usize;
    let band_data = &data[2..];

    if band_data.len() < band_count * BYTES_PER_BAND {
        return None;
    }

    let mut bands = Vec::with_capacity(band_count);
    for i in 0..band_count {
        let o = i * BYTES_PER_BAND;
        bands.push(BleEqBand {
            index: start_band + i as u8,
            gain: parse_ble_gain(band_data[o], band_data[o + 1]),
            frequency: decode_frequency(band_data[o + 2], band_data[o + 3]),
            q_value: decode_q(band_data[o + 4], band_data[o + 5]),
            filter_type: band_data[o + 6],
        });
    }
    Some(bands)
}

/// Encode a batch of EQ bands into a data payload for SET.
///
/// Output layout: `[start_band, end_band, {gain(2), freq(2), q(2), filter_type(1)} * N]`
pub fn encode_eq_band_batch(bands: &[BleEqBand]) -> Vec<u8> {
    if bands.is_empty() {
        return vec![0, 0];
    }

    let start_band = bands.first().unwrap().index;
    let end_band = bands.last().unwrap().index;

    let mut buf = Vec::with_capacity(2 + bands.len() * BYTES_PER_BAND);
    buf.push(start_band);
    buf.push(end_band);

    for band in bands {
        let (gh, gl) = encode_ble_gain(band.gain);
        let (fh, fl) = encode_frequency(band.frequency);
        let (qh, ql) = encode_q(band.q_value);
        buf.extend_from_slice(&[gh, gl, fh, fl, qh, ql, band.filter_type]);
    }
    buf
}

/// Parse EQ band batch response using the legacy per-band-with-index format.
///
/// Each band is 8 bytes: `[index, freq_hi, freq_lo, gain_hi, gain_lo, q_hi, q_lo, filter_type]`
pub fn parse_eq_band_response(data: &[u8]) -> Vec<BleEqBand> {
    let mut bands = Vec::new();
    let mut i = 0;
    while i + 8 <= data.len() {
        bands.push(BleEqBand {
            index: data[i],
            frequency: decode_frequency(data[i + 1], data[i + 2]),
            gain: parse_ble_gain(data[i + 3], data[i + 4]),
            q_value: decode_q(data[i + 5], data[i + 6]),
            filter_type: data[i + 7],
        });
        i += 8;
    }
    bands
}

// ===========================================================================
// GET command builders
// ===========================================================================

// ---- Device settings ----

/// GET volume level (0-99).
pub fn get_volume() -> Vec<u8> {
    build_packet(CMD_GET_VOLUME, &[])
}

/// GET gain mode (0=low, 1=high).
pub fn get_gain_mode() -> Vec<u8> {
    build_packet(CMD_GET_GAIN_MODE, &[])
}

/// GET channel balance `[direction, magnitude]`.
pub fn get_channel_balance() -> Vec<u8> {
    build_packet(CMD_GET_CHANNEL_BALANCE, &[])
}

/// GET SPDIF output switch.
pub fn get_spdif_out() -> Vec<u8> {
    build_packet(CMD_GET_SPDIF_OUT, &[])
}

/// GET DAC filter index.
pub fn get_dac_filter() -> Vec<u8> {
    build_packet(CMD_GET_DAC_FILTER, &[])
}

/// GET harmonic mode.
pub fn get_harmonic_mode() -> Vec<u8> {
    build_packet(CMD_GET_HARMONIC_MODE, &[])
}

/// GET auto power-off setting.
pub fn get_auto_power_off() -> Vec<u8> {
    build_packet(CMD_GET_AUTO_POWER_OFF, &[])
}

/// GET input source.
pub fn get_input_source() -> Vec<u8> {
    build_packet(CMD_GET_INPUT_SOURCE, &[])
}

// ---- EQ ----

/// GET EQ switch (on/off).
pub fn get_eq_switch() -> Vec<u8> {
    build_packet(CMD_GET_EQ_SWITCH, &[])
}

/// GET current EQ preset ID.
pub fn get_eq_preset() -> Vec<u8> {
    build_packet(CMD_GET_EQ_PRESET, &[])
}

/// GET EQ preset names (batch read). `start_idx` and `end_idx` are 0-based.
pub fn get_eq_preset_names(start_idx: u8, end_idx: u8) -> Vec<u8> {
    build_packet(CMD_GET_EQ_PRESET_NAMES, &[start_idx, end_idx])
}

/// GET EQ bands (batch). Retrieves bands from `start_band` to `end_band` inclusive.
pub fn get_eq_bands_batch(start_band: u8, end_band: u8) -> Vec<u8> {
    build_packet(CMD_GET_EQ_BAND_BATCH, &[start_band, end_band])
}

/// GET EQ preamp gain (signed i16 BE, dB * 10).
pub fn get_eq_preamp() -> Vec<u8> {
    build_packet(CMD_GET_EQ_PREAMP, &[])
}

// ---- Audio info ----

/// GET sample rate info `[dsd_flag, bit_depth, sample_rate_hi, sample_rate_lo]`.
pub fn get_sample_rate_info() -> Vec<u8> {
    build_packet(CMD_GET_SAMPLE_RATE_INFO, &[])
}

/// GET DAC chip-level filter index.
pub fn get_dac_chip_filter() -> Vec<u8> {
    build_packet(CMD_GET_DAC_CHIP_FILTER, &[])
}

// ---- Display ----

/// GET display mode (on/off).
pub fn get_display_mode() -> Vec<u8> {
    build_packet(CMD_GET_DISPLAY_MODE, &[])
}

/// GET screen brightness level.
pub fn get_screen_brightness() -> Vec<u8> {
    build_packet(CMD_GET_SCREEN_BRIGHTNESS, &[])
}

// ---- Lights (zone-parameterized) ----

/// GET light switch for a given zone.
pub fn get_light_switch(zone: u8) -> Vec<u8> {
    build_packet([0x05, 0x01, zone], &[])
}

/// GET light mode for a given zone.
pub fn get_light_mode(zone: u8) -> Vec<u8> {
    build_packet([0x05, 0x02, zone], &[])
}

/// GET light color for a given zone.
pub fn get_light_color(zone: u8) -> Vec<u8> {
    build_packet([0x05, 0x03, zone], &[])
}

// ---- Info (read-only) ----

/// GET firmware version (ASCII string response).
pub fn get_firmware_version() -> Vec<u8> {
    build_packet(CMD_GET_FIRMWARE_VERSION, &[])
}

/// GET current Bluetooth codec index.
pub fn get_bt_codec() -> Vec<u8> {
    build_packet(CMD_GET_BT_CODEC, &[])
}

// ===========================================================================
// SET command builders
// ===========================================================================

// ---- Device settings ----

/// SET volume level (0-99).
pub fn set_volume(level: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_VOLUME), &[level])
}

/// SET gain mode (0=low, 1=high).
pub fn set_gain_mode(mode: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_GAIN_MODE), &[mode])
}

/// SET channel balance. Raw `[direction, magnitude]` encoding.
/// Use `encode_channel_balance` to convert from a signed i16.
pub fn set_channel_balance(direction: u8, magnitude: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_CHANNEL_BALANCE), &[direction, magnitude])
}

/// SET channel balance from a signed value (positive = right, negative = left).
pub fn set_channel_balance_signed(balance: i16) -> Vec<u8> {
    let [dir, mag] = encode_channel_balance(balance);
    build_packet(get_to_set(CMD_GET_CHANNEL_BALANCE), &[dir, mag])
}

/// SET SPDIF output enabled/disabled.
pub fn set_spdif_out(enabled: bool) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_SPDIF_OUT), &[u8::from(enabled)])
}

/// SET DAC filter by index.
pub fn set_dac_filter(index: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_DAC_FILTER), &[index])
}

/// SET harmonic mode.
pub fn set_harmonic_mode(mode: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_HARMONIC_MODE), &[mode])
}

/// SET auto power-off timeout.
pub fn set_auto_power_off(value: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_AUTO_POWER_OFF), &[value])
}

/// SET input source.
pub fn set_input_source(source: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_INPUT_SOURCE), &[source])
}

// ---- EQ ----

/// SET EQ switch on/off.
pub fn set_eq_switch(enabled: bool) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_EQ_SWITCH), &[u8::from(enabled)])
}

/// SET EQ preset by ID.
pub fn set_eq_preset(preset: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_EQ_PRESET), &[preset])
}

/// SET EQ bands (batch). Encodes the provided bands using the batch format.
pub fn set_eq_bands_batch(bands: &[BleEqBand]) -> Vec<u8> {
    let data = encode_eq_band_batch(bands);
    build_packet(get_to_set(CMD_GET_EQ_BAND_BATCH), &data)
}

/// SET a single EQ band. Convenience wrapper that encodes one band as a batch.
pub fn set_eq_band_single(index: u8, freq: u16, gain: f64, q: f64, filter_type: u8) -> Vec<u8> {
    let band = BleEqBand {
        index,
        frequency: freq,
        gain,
        q_value: q,
        filter_type,
    };
    set_eq_bands_batch(&[band])
}

/// SET all EQ bands (convenience alias for `set_eq_bands_batch`).
pub fn set_all_eq_bands(bands: &[BleEqBand]) -> Vec<u8> {
    set_eq_bands_batch(bands)
}

/// SET EQ preamp gain in dB (e.g. -12.0, +6.0). Encoded as signed i16 BE * 10.
pub fn set_eq_preamp(gain_db: f64) -> Vec<u8> {
    let (hi, lo) = encode_ble_gain(gain_db);
    build_packet(get_to_set(CMD_GET_EQ_PREAMP), &[hi, lo])
}

// ---- Audio / DAC ----

/// SET DAC chip-level filter.
pub fn set_dac_chip_filter(index: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_DAC_CHIP_FILTER), &[index])
}

// ---- Display ----

/// SET display mode on/off.
pub fn set_display_mode(on: bool) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_DISPLAY_MODE), &[u8::from(on)])
}

/// SET screen brightness level.
pub fn set_screen_brightness(level: u8) -> Vec<u8> {
    build_packet(get_to_set(CMD_GET_SCREEN_BRIGHTNESS), &[level])
}

// ---- Lights ----

/// SET light switch for a given zone.
pub fn set_light_switch(zone: u8, on: bool) -> Vec<u8> {
    build_packet(get_to_set([0x05, 0x01, zone]), &[u8::from(on)])
}

/// SET light mode for a given zone.
pub fn set_light_mode(zone: u8, mode: u8) -> Vec<u8> {
    build_packet(get_to_set([0x05, 0x02, zone]), &[mode])
}

/// SET light color for a given zone.
pub fn set_light_color(zone: u8, color: u8) -> Vec<u8> {
    build_packet(get_to_set([0x05, 0x03, zone]), &[color])
}

// ===========================================================================
// Lookup helpers
// ===========================================================================

/// Get the human-readable name for a Bluetooth codec index.
pub fn bt_codec_name(index: u8) -> &'static str {
    BT_CODEC_NAMES.get(index as usize).unwrap_or(&"Unknown")
}

/// Get the human-readable name for an EQ filter type.
pub fn filter_type_name(ft: u8) -> &'static str {
    FILTER_TYPE_NAMES.get(ft as usize).unwrap_or(&"Unknown")
}

/// Get the human-readable name for a DAC filter index.
pub fn dac_filter_name(index: u8) -> &'static str {
    DAC_FILTER_NAMES.get(index as usize).unwrap_or(&"Unknown")
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_packet_no_data() {
        let pkt = build_packet([0x02, 0x01, 0x01], &[]);
        assert_eq!(pkt, vec![0xF1, 0x10, 0x00, 0x08, 0x02, 0x01, 0x01, 0xFF]);
    }

    #[test]
    fn test_build_packet_with_data() {
        let pkt = build_packet([0x12, 0x01, 0x01], &[0x42]);
        assert_eq!(pkt, vec![0xF1, 0x10, 0x00, 0x09, 0x12, 0x01, 0x01, 0x42, 0xFF]);
    }

    #[test]
    fn test_get_to_set() {
        assert_eq!(get_to_set([0x02, 0x01, 0x01]), [0x12, 0x01, 0x01]);
        assert_eq!(get_to_set([0x03, 0x0C, 0x01]), [0x13, 0x0C, 0x01]);
        assert_eq!(get_to_set([0x05, 0x01, 0x01]), [0x15, 0x01, 0x01]);
    }

    #[test]
    fn test_parse_response() {
        let raw = vec![0xF1, 0x10, 0x00, 0x09, 0x02, 0x01, 0x01, 0x42, 0xFF];
        let (cmd, data) = parse_response(&raw).unwrap();
        assert_eq!(cmd, [0x02, 0x01, 0x01]);
        assert_eq!(data, vec![0x42]);
    }

    #[test]
    fn test_gain_encode_decode() {
        let (h, l) = encode_ble_gain(6.5);
        assert_eq!(parse_ble_gain(h, l), 6.5);

        let (h, l) = encode_ble_gain(-12.0);
        assert_eq!(parse_ble_gain(h, l), -12.0);

        let (h, l) = encode_ble_gain(0.0);
        assert_eq!(parse_ble_gain(h, l), 0.0);
    }

    #[test]
    fn test_q_encode_decode() {
        let (h, l) = encode_q(1.41);
        assert_eq!(decode_q(h, l), 1.41);

        let (h, l) = encode_q(0.71);
        assert_eq!(decode_q(h, l), 0.71);
    }

    #[test]
    fn test_channel_balance() {
        assert_eq!(encode_channel_balance(0), [0x00, 0x00]);
        assert_eq!(parse_channel_balance(&[0x00, 0x00]), Some(0));

        assert_eq!(encode_channel_balance(5), [BALANCE_RIGHT, 5]);
        assert_eq!(parse_channel_balance(&[BALANCE_RIGHT, 5]), Some(5));

        assert_eq!(encode_channel_balance(-3), [BALANCE_LEFT, 3]);
        assert_eq!(parse_channel_balance(&[BALANCE_LEFT, 3]), Some(-3));
    }

    #[test]
    fn test_eq_band_batch_roundtrip() {
        let bands = vec![
            BleEqBand { index: 0, frequency: 100, gain: -3.5, q_value: 1.41, filter_type: FILTER_PEAK },
            BleEqBand { index: 1, frequency: 1000, gain: 6.0, q_value: 0.71, filter_type: FILTER_HIGH_SHELF },
        ];
        let encoded = encode_eq_band_batch(&bands);
        let decoded = decode_eq_band_batch(&encoded).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].index, 0);
        assert_eq!(decoded[0].frequency, 100);
        assert_eq!(decoded[0].gain, -3.5);
        assert_eq!(decoded[0].q_value, 1.41);
        assert_eq!(decoded[0].filter_type, FILTER_PEAK);

        assert_eq!(decoded[1].index, 1);
        assert_eq!(decoded[1].frequency, 1000);
        assert_eq!(decoded[1].gain, 6.0);
        assert_eq!(decoded[1].q_value, 0.71);
        assert_eq!(decoded[1].filter_type, FILTER_HIGH_SHELF);
    }

    #[test]
    fn test_set_volume_packet() {
        let pkt = set_volume(50);
        // SET volume: cmd = [0x12, 0x01, 0x01], data = [50]
        assert_eq!(pkt, vec![0xF1, 0x10, 0x00, 0x09, 0x12, 0x01, 0x01, 50, 0xFF]);
    }

    #[test]
    fn test_set_eq_preamp_packet() {
        let pkt = set_eq_preamp(-6.0);
        // -60 as i16 BE = [0xFF, 0xC4]
        assert_eq!(&pkt[7..9], &[0xFF, 0xC4]);
    }

    #[test]
    fn test_sample_rate_info() {
        let info = parse_sample_rate_info(&[0x00, 24, 0xAC, 0x44]).unwrap();
        assert!(!info.is_dsd);
        assert_eq!(info.bit_depth, 24);
        assert_eq!(info.sample_rate, 44100);
    }

    #[test]
    fn test_command_triplet_set_equivalents() {
        // Verify all command categories produce correct SET triplets
        assert_eq!(get_to_set(CMD_GET_VOLUME), [0x12, 0x01, 0x01]);
        assert_eq!(get_to_set(CMD_GET_GAIN_MODE), [0x12, 0x02, 0x01]);
        assert_eq!(get_to_set(CMD_GET_CHANNEL_BALANCE), [0x12, 0x06, 0x01]);
        assert_eq!(get_to_set(CMD_GET_SPDIF_OUT), [0x12, 0x08, 0x01]);
        assert_eq!(get_to_set(CMD_GET_DAC_FILTER), [0x12, 0x09, 0x01]);
        assert_eq!(get_to_set(CMD_GET_HARMONIC_MODE), [0x12, 0x0A, 0x01]);
        assert_eq!(get_to_set(CMD_GET_AUTO_POWER_OFF), [0x12, 0x0B, 0x01]);
        assert_eq!(get_to_set(CMD_GET_EQ_SWITCH), [0x13, 0x01, 0x01]);
        assert_eq!(get_to_set(CMD_GET_EQ_PRESET), [0x13, 0x03, 0x01]);
        assert_eq!(get_to_set(CMD_GET_EQ_BAND_BATCH), [0x13, 0x0C, 0x01]);
        assert_eq!(get_to_set(CMD_GET_EQ_PREAMP), [0x13, 0x0D, 0x01]);
        assert_eq!(get_to_set(CMD_GET_DAC_CHIP_FILTER), [0x14, 0x05, 0x01]);
        assert_eq!(get_to_set(CMD_GET_DISPLAY_MODE), [0x15, 0x01, 0x01]);
        assert_eq!(get_to_set(CMD_GET_SCREEN_BRIGHTNESS), [0x15, 0x05, 0x01]);
    }
}
