<p align="center">
  <h1 align="center">FiiO K13 R2R Control</h1>
  <p align="center">Desktop controller for FiiO K13 R2R DAC/AMP with full EQ management and AutoEQ integration.</p>
  <p align="center">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    <img src="https://img.shields.io/badge/tauri-2-blue.svg" alt="Tauri">
    <img src="https://img.shields.io/badge/svelte-5-orange.svg" alt="Svelte">
    <img src="https://img.shields.io/badge/rust-1.92-red.svg" alt="Rust">
    <img src="https://img.shields.io/badge/platform-linux-lightgrey.svg" alt="Platform">
    <img src="https://img.shields.io/badge/status-Active-green.svg" alt="Status">
  </p>
</p>

---

## Table of Contents

- [About](#about)
- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Technical Details](#technical-details)

## About

**FiiO K13 R2R Control** is a native desktop application for controlling the FiiO K13 R2R DAC/AMP via USB HID. Built by reverse-engineering the [FiiO Control web app](https://fiiocontrol.fiio.com), it communicates directly with the device over USB — no Bluetooth required.

The app provides a full 10-band parametric EQ editor, preset management, and one-click headphone correction from the [AutoEQ](https://github.com/jaakkopasanen/AutoEq) database.

## Features

- **10-Band Parametric EQ** — Full control over frequency (20Hz–20kHz), gain (±12dB), Q factor (0.1–10.0), and 7 filter types (Peak, Low Shelf, High Shelf, Band Pass, Low Pass, High Pass, All Pass)
- **Live Frequency Response** — Real-time visualization of the EQ curve as you edit
- **Preset Management** — Switch between factory presets (Jazz, Pop, Rock, Dance, R&B, Classic, HipHop, Retro, sDamp) and 10 user-editable slots with custom names
- **AutoEQ Integration** — Browse thousands of headphone correction profiles, preview the curve, and apply with one click
- **Global Gain Control** — Adjust the overall EQ preamp level
- **Direct USB Communication** — No BLE, no middleware — raw USB HID interrupt transfers via `rusb`
- **Save to Device** — Persist settings directly to the K13 R2R's onboard memory

## Requirements

- **OS**: Linux (tested on Fedora 43). Windows/macOS support is possible but untested.
- **Device**: FiiO K13 R2R connected via USB
- **Build tools**: Rust 1.70+, Node.js 18+, Bun

> **USB permissions**: You may need a udev rule to access the device without root. Create `/etc/udev/rules.d/99-fiio.rules`:
> ```
> SUBSYSTEM=="usb", ATTR{idVendor}=="2972", MODE="0666"
> ```
> Then run `sudo udevadm control --reload-rules && sudo udevadm trigger`.

## Installation

### From Source

```bash
git clone https://github.com/SmookeyDev/fiio-k13-control.git
cd fiio-k13-control
bun install
```

#### Build dependencies

```bash
sudo dnf install rust cargo webkit2gtk4.1-devel libusb1-devel libappindicator-gtk3-devel
```

#### Development

```bash
bun run tauri dev
```

#### Production build

```bash
bun run tauri build
```

## Usage

1. **Connect** the K13 R2R via USB and launch the app
2. Click **Connect** — the device name appears in the sidebar
3. **Edit EQ bands** by selecting a band and adjusting frequency, gain, Q, and filter type
4. **Switch presets** from the dropdown — factory or user slots
5. **Save** your settings to any of the 10 user preset slots
6. For headphone correction, go to **AutoEQ**, search your model, and click **Apply**

### Presets

- **Factory** (read-only): Jazz, Pop, Rock, Dance, R&B, Classic, HipHop, Retro, sDamp-1, sDamp-2
- **User** (editable): USER 1–10, with renameable labels
- **Bypass**: Disables EQ processing entirely

## Technical Details

### USB HID Protocol

The K13 R2R communicates over USB HID on **interface 3** using interrupt transfers (`EP OUT 0x02`, `EP IN 0x83`, Report ID `0x07`).

Packets follow a fixed structure:

```
TX: [0xAA] [DIR] [00] [LEN] [CMD] [SUB] [DATA...] [zero-padded to 64 bytes]
RX: [0xBB] [DIR] [00] [LEN] [CMD] [SUB] [DATA...]
```

- `DIR` — `0x01` = GET (read from device), `0x02` = SET (write to device)
- `CMD` — Command ID (e.g., `0x07` = EQ band, `0x09` = preset selection)
- `SUB` — Sub-command or band index

### Data Encoding

- **Frequency**: 2 bytes big-endian, value in Hz
- **Gain**: 2 bytes — `[sign_byte][abs_value × 10]`, where sign `0x01` = negative
- **Q Factor**: 2 bytes — `[integer_part][decimal × 10]`
- **Filter Type**: 1 byte — `0`=Peak, `1`=LSC, `2`=HSC, `3`=BPF, `4`=LPF, `5`=HPF, `6`=APF

### Architecture

```
┌─────────────────────────────┐
│  Svelte 5 Frontend          │
│  (Runes + SvelteKit)        │
│         │                   │
│    invoke() ──► Tauri IPC   │
└─────────┬───────────────────┘
          │
┌─────────▼───────────────────┐
│  Rust Backend               │
│  commands.rs → device.rs    │
│         │                   │
│    rusb ──► USB HID         │
└─────────┬───────────────────┘
          │
┌─────────▼───────────────────┐
│  FiiO K13 R2R               │
│  (Interface 3, EP 0x02/83)  │
└─────────────────────────────┘
```

### AutoEQ Pipeline

1. Fetches the [AutoEQ INDEX.md](https://github.com/jaakkopasanen/AutoEq) and parses the headphone list
2. On selection, fetches the `ParametricEQ.txt` for that model
3. Parses preamp, frequency, gain, Q, and filter type for each band
4. Writes each band to the device via USB HID and saves to the selected user preset slot

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

