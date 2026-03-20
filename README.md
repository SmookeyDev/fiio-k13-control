<p align="center">
  <h1 align="center">FiiO K13 R2R Control</h1>
  <p align="center">Desktop controller for FiiO K13 R2R DAC/AMP — USB HID + Bluetooth LE.</p>
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
- [BLE Setup (Linux)](#ble-setup-linux)
- [Installation](#installation)
- [Usage](#usage)
- [Technical Details](#technical-details)
- [Known Issues](#known-issues)

## About

**FiiO K13 R2R Control** is a native desktop application for controlling the FiiO K13 R2R DAC/AMP. It communicates with the device over **USB HID** (for EQ/preset management) and **Bluetooth Low Energy** (for input source, indicator lights, and device settings).

The BLE protocol was reverse-engineered from the official [FiiO Control](https://play.google.com/store/apps/details?id=com.fiio.control) Android APK (v4.0.3).

## Features

### USB HID
- **10-Band Parametric EQ** — Frequency (20Hz–20kHz), gain (-24/+12dB), Q (0.1–10.0), 7 filter types
- **Live Frequency Response** — Real-time EQ curve visualization
- **Preset Management** — Factory presets + 10 user slots with custom names
- **AutoEQ Integration** — One-click headphone correction from the [AutoEQ](https://github.com/jaakkopasanen/AutoEq) database
- **Global Pre-Amp** — Overall EQ gain adjustment
- **Save to Device** — Persist settings to onboard memory

### Bluetooth LE
- **Input Source** — Switch between USB, Coaxial, Optical, Bluetooth
- **Indicator Lights** — Control top and knob LEDs (on/off, color, mode)
- **Device Info** — Firmware version readout
- **Auto-retry** — Automatic connection retries with full BlueZ cache cleanup

## Requirements

- **OS**: Linux (tested on Fedora 43)
- **Device**: FiiO K13 R2R
- **Build tools**: Rust 1.70+, Node.js 18+, Bun

> **USB permissions**: Create `/etc/udev/rules.d/99-fiio.rules`:
> ```
> SUBSYSTEM=="usb", ATTR{idVendor}=="2972", MODE="0666"
> ```
> Then run `sudo udevadm control --reload-rules && sudo udevadm trigger`.

## BLE Setup (Linux)

The K13 R2R is a dual-mode Bluetooth device (Classic + BLE). On Linux with **Intel AX210** (and similar Intel adapters), BlueZ cannot connect BLE to dual-mode devices in the default `ControllerMode = dual`.

### Required: Set ControllerMode to LE

Edit `/etc/bluetooth/main.conf`:

```ini
# Change from:
#ControllerMode = dual

# To:
ControllerMode = le
```

Then restart Bluetooth:

```bash
sudo systemctl restart bluetooth
```

> **Trade-off**: This disables Bluetooth Classic (A2DP audio, headset profiles). If you need Classic BT for audio devices, switch back to `dual` when not using BLE.

### Why is this needed?

This is a [known BlueZ issue](https://github.com/bluez/bluez/issues/577) with Intel adapters. In `dual` mode, the controller's LE Coded PHY support interferes with BLE connections to dual-mode devices. The kernel has a `HCI_QUIRK_BROKEN_LE_CODED` quirk for this, but it doesn't fully resolve the issue for all devices.

### BLE Connection Behavior

- The app uses D-Bus with an explicit LE transport filter to scan, then `bluetoothctl` to connect
- Connection may take 2-3 automatic retries (the app handles this internally)
- If the connection drops, the app detects it and updates the UI accordingly
- The K13 BLE address is `41:42:F9:EB:E1:35` (BLE) vs `41:42:F9:EB:E1:60` (Classic/A2DP)

## Installation

### From Source

```bash
git clone https://github.com/SmookeyDev/fiio-k13-control.git
cd fiio-k13-control
bun install
```

#### Build dependencies

```bash
# Fedora
sudo dnf install rust cargo webkit2gtk4.1-devel libusb1-devel libappindicator-gtk3-devel dbus-devel

# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev libusb-1.0-0-dev libappindicator3-dev libdbus-1-dev
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

1. **Connect USB** — Click "Connect USB" for EQ management
2. **Connect BLE** — Click "Connect BLE" for input source and light control
3. **Edit EQ bands** — Select a band, adjust frequency/gain/Q/filter type
4. **Switch presets** — Factory or user slots from the dropdown
5. **Save** settings to any of the 10 user preset slots
6. **AutoEQ** — Search your headphone model and apply correction with one click
7. **Settings** (BLE) — Change input source, toggle indicator lights, pick LED colors

### Presets

- **Factory** (read-only): Jazz, Pop, Rock, Dance, R&B, Classic, HipHop, Retro, sDamp-1, sDamp-2
- **User** (editable): USER 1–10, with renameable labels
- **Bypass**: Disables EQ processing entirely

## Technical Details

### USB HID Protocol

The K13 R2R communicates over USB HID on **interface 3** using interrupt transfers (`EP OUT 0x02`, `EP IN 0x83`, Report ID `0x07`).

```
GET: [0xBB, 0x0B, 0x00, 0x00, CMD, DATA_LEN, ...DATA, 0x00, 0xEE]
SET: [0xAA, 0x0A, 0x00, 0x00, CMD, DATA_LEN, ...DATA, 0x00, 0xEE]
```

### BLE Protocol

BLE uses GATT service `00001100-04a5-1000-1000-40ed981a04a5` with write (`1101`) and notify (`1102`) characteristics.

```
Packet: [0xF1, 0x10, 0x00, LEN, CMD0, CMD1, CMD2, DATA..., 0xFF]
GET: CMD0 high nibble = 0x0X
SET: CMD0 high nibble = 0x1X (CMD0 | 0x10)
```

### Data Encoding

- **Frequency**: unsigned 16-bit BE, value in Hz
- **Gain**: signed 16-bit BE, value × 10 (e.g., -2.5 dB = `0xFFE7`)
- **Q Factor**: unsigned 16-bit BE, value × 100 (e.g., 0.71 = `0x0047`)
- **Filter Type**: 1 byte — `0`=Peak, `1`=Low Shelf, `2`=High Shelf, `3`=Band Pass, `4`=Low Pass, `5`=High Pass, `6`=All Pass

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
│  commands.rs                │
│    ├── device.rs ──► USB    │
│    └── ble_device.rs ──► BLE│
└─────────┬──────┬────────────┘
          │      │
┌─────────▼──┐ ┌─▼──────────────┐
│ USB HID    │ │ BLE (btleplug) │
│ (rusb)     │ │ + D-Bus/BlueZ  │
│ EP 0x02/83 │ │ GATT 1100-04a5 │
└────────────┘ └────────────────┘
```

## Known Issues

| Issue | Cause | Workaround |
|-------|-------|------------|
| BLE connect fails in `ControllerMode = dual` | Intel AX210 + BlueZ bug with dual-mode BLE devices ([bluez#577](https://github.com/bluez/bluez/issues/577)) | Set `ControllerMode = le` in `/etc/bluetooth/main.conf` |
| BLE needs 2-3 retries to connect | BlueZ GATT cache stale after failed attempts | App auto-retries with `bluetoothctl remove` between attempts |
| BLE connection drops after rapid commands | K13 firmware disconnects on command flood | App uses debounce and drains ACK notifications |
| Classic BT audio unavailable | `ControllerMode = le` disables BR/EDR | Switch back to `dual` when not using BLE |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
