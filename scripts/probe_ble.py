#!/usr/bin/env python3
"""
FiiO K13 R2R — BLE Feature Probe
Confirms all BLE commands discovered from reverse-engineering the Android app.

Usage: python3 scripts/probe_ble.py
"""

import asyncio
import sys
import os
from datetime import datetime

try:
    from bleak import BleakClient, BleakScanner
except ImportError:
    print("Install bleak: pip3 install bleak")
    sys.exit(1)

# BLE UUIDs
SERVICE_UUID = "00001100-04a5-1000-1000-40ed981a04a5"
WRITE_UUID = "00001101-04a5-1000-1000-40ed981a04a5"
NOTIFY_UUID = "00001102-04a5-1000-1000-40ed981a04a5"

# Commands to probe (GET only — no SET to avoid changing device state)
COMMANDS = [
    # Confirmed for K13 R2R (device type 43)
    ("Input Source", bytes([0x09, 0x02, 0x01])),
    ("Top Light On/Off", bytes([0x05, 0x01, 0x02])),
    ("Top Light Mode", bytes([0x05, 0x02, 0x02])),
    ("Top Light Color", bytes([0x05, 0x03, 0x02])),
    ("Knob Light On/Off", bytes([0x05, 0x01, 0x03])),
    ("Knob Light Mode", bytes([0x05, 0x02, 0x03])),
    ("Knob Light Color", bytes([0x05, 0x03, 0x03])),

    # Inferred from same device family
    ("Firmware Version", bytes([0x00, 0x00, 0x02])),
    ("Battery/Power", bytes([0x00, 0x00, 0x03])),
    ("Screen/Display Mode", bytes([0x00, 0x02, 0x01])),
    ("Volume", bytes([0x00, 0x02, 0x02])),
    ("Channel Balance", bytes([0x00, 0x02, 0x06])),
    ("De-emphasis", bytes([0x00, 0x02, 0x08])),
    ("BT Connection/Codec", bytes([0x00, 0x03, 0x01])),
    ("PEQ Band Data", bytes([0x00, 0x03, 0x02])),
    ("EQ Preset Index", bytes([0x00, 0x03, 0x04])),
    ("DAC Filter Mode", bytes([0x00, 0x05, 0x01])),
    ("DAC Filter Mode 2", bytes([0x00, 0x05, 0x02])),
    ("DAC Filter Mode 3", bytes([0x00, 0x05, 0x03])),
    ("Device Name", bytes([0x00, 0x08, 0x01])),
    ("Audio Codec Display", bytes([0x00, 0x08, 0x02])),
    ("EQ Settings", bytes([0x00, 0x09, 0x02])),
    ("Gain Mode", bytes([0x00, 0x02, 0x03])),
    ("Vol Max", bytes([0x00, 0x02, 0x04])),
    ("Vol Output Switch", bytes([0x00, 0x02, 0x05])),
    ("Screen Orientation", bytes([0x00, 0x02, 0x07])),

    # Extra exploration
    ("Unknown 0x000001", bytes([0x00, 0x00, 0x01])),
    ("Unknown 0x000004", bytes([0x00, 0x00, 0x04])),
    ("Unknown 0x000101", bytes([0x00, 0x01, 0x01])),
    ("Unknown 0x000102", bytes([0x00, 0x01, 0x02])),
    ("Unknown 0x000401", bytes([0x00, 0x04, 0x01])),
    ("Unknown 0x000601", bytes([0x00, 0x06, 0x01])),
    ("Unknown 0x000701", bytes([0x00, 0x07, 0x01])),
]

INPUT_SOURCES = {0x01: "USB", 0x04: "COAXIAL", 0x08: "OPTICAL", 0x20: "BLUETOOTH"}
LIGHT_COLORS = {
    0x00: "follow_audio", 0x01: "red", 0x02: "blue", 0x03: "turquoise",
    0x04: "purple", 0x05: "yellow", 0x06: "white", 0x07: "green", 0x08: "cycle",
}
LIGHT_MODES = {0x00: "always_on", 0x01: "breathe"}


def build_ble_packet(cmd: bytes, data: bytes = b"") -> bytes:
    payload = cmd + data
    length = len(payload) + 5  # F1 10 00 LEN ... FF
    pkt = bytearray([0xF1, 0x10, 0x00, length])
    pkt.extend(payload)
    pkt.append(0xFF)
    return bytes(pkt)


def hex_str(data: bytes) -> str:
    return " ".join(f"{b:02X}" for b in data)


def decode_response(name: str, cmd: bytes, data: bytes) -> list[str]:
    info = []
    if not data:
        return info

    cmd_hex = hex_str(cmd)

    if "Input Source" in name and len(data) >= 1:
        src = data[0]
        sources = []
        for bit, label in INPUT_SOURCES.items():
            if src & bit:
                sources.append(label)
        info.append(f"Active: {', '.join(sources) if sources else f'0x{src:02X}'}")

    elif "Light On/Off" in name and len(data) >= 1:
        info.append(f"{'ON' if data[0] else 'OFF'}")

    elif "Light Mode" in name and len(data) >= 1:
        info.append(f"{LIGHT_MODES.get(data[0], f'0x{data[0]:02X}')}")

    elif "Light Color" in name and len(data) >= 1:
        info.append(f"{LIGHT_COLORS.get(data[0], f'0x{data[0]:02X}')}")

    elif "Firmware" in name:
        if len(data) >= 2:
            info.append(f"v{data[0]}.{data[1]}")
        else:
            info.append(f"v{data[0]}")

    elif "Volume" in name and len(data) >= 1:
        info.append(f"Level: {data[0]}")

    elif "Channel Balance" in name and len(data) >= 2:
        direction = "right" if data[0] == 0x01 else "left"
        info.append(f"{direction} +{data[1]}")

    elif "Battery" in name and len(data) >= 1:
        info.append(f"{data[0]}%")

    elif "DAC Filter" in name and len(data) >= 1:
        info.append(f"Filter index: {data[0]}")

    elif "Gain" in name and len(data) >= 1:
        info.append(f"Mode: {data[0]}")

    elif "Device Name" in name:
        text = bytes(data).decode("utf-8", errors="replace").rstrip("\x00\xff")
        if text:
            info.append(f"'{text}'")

    elif "Audio Codec" in name and len(data) >= 1:
        codecs = ["PCM", "DSD", "MQA", "DXD"]
        idx = data[0]
        info.append(f"{codecs[idx] if idx < len(codecs) else f'0x{idx:02X}'}")

    if not info:
        vals = " ".join(f"0x{b:02X}" for b in data)
        info.append(f"Raw: {vals}")
        text = bytes(data).decode("utf-8", errors="replace").rstrip("\x00\xff")
        if text and all(32 <= ord(c) < 127 for c in text):
            info.append(f"Text: '{text}'")

    return info


def parse_response(raw: bytes) -> tuple[bytes, bytes] | None:
    """Parse BLE response, return (cmd, data) or None."""
    if len(raw) < 5:
        return None
    if raw[0] != 0xF1 or raw[1] != 0x10:
        return None
    cmd = bytes(raw[4:7]) if len(raw) >= 7 else b""
    data = bytes(raw[7:-1]) if len(raw) > 8 and raw[-1] == 0xFF else bytes(raw[7:])
    return cmd, data


class BleProbe:
    def __init__(self):
        self.response = None
        self.response_event = asyncio.Event()

    def notification_handler(self, sender, data: bytearray):
        self.response = bytes(data)
        self.response_event.set()

    async def send_and_receive(self, client: BleakClient, name: str, cmd: bytes, timeout: float = 2.0) -> tuple[bytes, bytes] | None:
        self.response = None
        self.response_event.clear()

        pkt = build_ble_packet(cmd)
        await client.write_gatt_char(WRITE_UUID, pkt, response=False)

        try:
            await asyncio.wait_for(self.response_event.wait(), timeout)
        except asyncio.TimeoutError:
            return None

        if self.response is None:
            return None

        return parse_response(self.response)


async def main():
    log_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "logs")
    try:
        os.makedirs(log_dir, exist_ok=True)
        log_path = os.path.join(log_dir, f"ble_probe_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log")
        open(log_path, "w").close()  # test write
    except PermissionError:
        log_dir = "/tmp"
        log_path = os.path.join(log_dir, f"ble_probe_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log")

    class Tee:
        def __init__(self, *streams):
            self.streams = streams
        def write(self, data):
            for s in self.streams:
                s.write(data)
                s.flush()
        def flush(self):
            for s in self.streams:
                s.flush()

    log_file = open(log_path, "w")
    sys.stdout = Tee(sys.__stdout__, log_file)

    print("=" * 70)
    print("FiiO K13 R2R — BLE Feature Probe")
    print(f"Log: {log_path}")
    print("=" * 70)
    print()

    # Scan for K13 R2R
    print("Scanning for FiiO K13 R2R...")
    device = None
    devices = await BleakScanner.discover(timeout=10.0)
    for d in devices:
        name = d.name or ""
        if "k13" in name.lower() or "fiio" in name.lower():
            print(f"  Found: {d.name} ({d.address})")
            device = d
            break

    if device is None:
        # Try finding by service UUID
        for d in devices:
            for ad in (d.metadata.get("uuids") or []):
                if "04a5" in ad.lower():
                    print(f"  Found by UUID: {d.name} ({d.address})")
                    device = d
                    break
            if device:
                break

    if device is None:
        print("\nERROR: K13 R2R not found. Is Bluetooth enabled and device in range?")
        print(f"Visible devices ({len(devices)}):")
        for d in devices:
            print(f"  {d.name or '???'} — {d.address}")
        log_file.close()
        sys.exit(1)

    print(f"\nConnecting to {device.name} ({device.address})...")

    async with BleakClient(device, timeout=30.0) as client:
        print(f"Connected: {client.is_connected}")
        print()

        # List services
        print("Services:")
        for service in client.services:
            print(f"  {service.uuid} — {service.description}")
            for char in service.characteristics:
                props = ", ".join(char.properties)
                print(f"    {char.uuid} [{props}]")
        print()

        # Setup notifications
        probe = BleProbe()
        await client.start_notify(NOTIFY_UUID, probe.notification_handler)

        # Probe all commands
        print("-" * 70)
        print("PROBING COMMANDS")
        print("-" * 70)
        print()

        results = {}

        for name, cmd in COMMANDS:
            cmd_hex = hex_str(cmd)
            sys.stdout.write(f"  {name:30s} [{cmd_hex}] ... ")
            sys.stdout.flush()

            result = await probe.send_and_receive(client, name, cmd)

            if result is None:
                print("NO RESPONSE")
                results[name] = None
            else:
                resp_cmd, data = result
                print(f"OK ({len(data)}B)")
                print(f"    Raw response: {hex_str(probe.response)}")
                if data:
                    print(f"    Data: {hex_str(data)}")
                    decoded = decode_response(name, cmd, data)
                    for line in decoded:
                        print(f"    >>> {line}")
                results[name] = (resp_cmd, data)

            await asyncio.sleep(0.1)

        await client.stop_notify(NOTIFY_UUID)

    # Summary
    print()
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print()

    working = [(n, r) for n, r in results.items() if r is not None]
    no_response = [n for n, r in results.items() if r is None]

    print(f"Working commands: {len(working)}/{len(results)}")
    print()
    for name, (cmd, data) in working:
        data_hex = hex_str(data) if data else "(empty)"
        decoded = decode_response(name, cmd, data)
        decoded_str = " | ".join(decoded) if decoded else ""
        print(f"  {name:30s} {data_hex:30s} {decoded_str}")

    if no_response:
        print(f"\nNo response ({len(no_response)}):")
        for name in no_response:
            print(f"  {name}")

    print()
    print("=" * 70)
    print("Probe complete.")
    print("=" * 70)

    log_file.close()


if __name__ == "__main__":
    asyncio.run(main())
