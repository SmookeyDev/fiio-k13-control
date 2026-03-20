#!/usr/bin/env python3
"""
FiiO K13 R2R — USB HID Feature Probe v3
Scans all GET commands, probes sub-values safely, and attempts firmware info extraction.

Usage: sudo python3 scripts/probe_features.py
"""

import usb.core
import usb.util
import time
import sys
import os
from datetime import datetime

VENDOR_ID = 0x2972
INTERFACE = 3
EP_OUT = 0x02
EP_IN = 0x83
REPORT_ID = 0x07
TIMEOUT_MS = 300

# Known EQ commands (safe for sub-value probing)
SAFE_FOR_SUB_PROBE = {0x15, 0x16, 0x17, 0x18, 0x1A, 0x1B, 0x30}

KNOWN_CMDS = {
    0x15: "EQ Band Item", 0x16: "EQ Preset", 0x17: "EQ Global Gain",
    0x18: "EQ Count", 0x19: "EQ Save", 0x1A: "EQ Switch",
    0x1B: "EQ Reset", 0x30: "Preset Name",
}


class DeviceDead(Exception):
    pass


def build_get_packet(cmd, data=b""):
    pkt = bytearray([0xBB, 0x0B, 0x00, 0x00, cmd, len(data)])
    pkt.extend(data)
    pkt.append(0x00)
    pkt.append(0xEE)
    return bytes(pkt)


def build_usb_frame(packet):
    buf = bytearray(65)
    buf[0] = REPORT_ID
    n = min(len(packet), 64)
    buf[1:1 + n] = packet[:n]
    return bytes(buf)


def drain(dev):
    try:
        while True:
            dev.read(EP_IN, 65, timeout=30)
    except (usb.core.USBTimeoutError, usb.core.USBError):
        pass


def send_get(dev, cmd, data=b"", io_errors=None):
    """Send GET. Returns response bytes or None. Tracks consecutive I/O errors."""
    try:
        drain(dev)
        frame = build_usb_frame(build_get_packet(cmd, data))
        dev.write(EP_OUT, frame, timeout=500)

        for _ in range(2):
            try:
                resp = bytes(dev.read(EP_IN, 65, timeout=TIMEOUT_MS))
                if resp and resp[0] == REPORT_ID:
                    resp = resp[1:]
                if len(resp) > 4 and resp[0] == 0xBB and resp[4] == cmd:
                    if io_errors is not None:
                        io_errors[0] = 0
                    return resp
            except usb.core.USBTimeoutError:
                return None
        return None
    except usb.core.USBError as e:
        if io_errors is not None:
            io_errors[0] += 1
            if io_errors[0] >= 3:
                raise DeviceDead(f"3 consecutive I/O errors (last: {e})")
        return None


def health_check(dev):
    """Verify device is still alive by reading EQ count."""
    try:
        drain(dev)
        frame = build_usb_frame(build_get_packet(0x18))
        dev.write(EP_OUT, frame, timeout=500)
        resp = bytes(dev.read(EP_IN, 65, timeout=500))
        return len(resp) > 0
    except (usb.core.USBError, usb.core.USBTimeoutError):
        return False


def hex_str(data, limit=32):
    s = " ".join(f"{b:02X}" for b in data[:limit])
    if len(data) > limit:
        s += " ..."
    return s


def parse_gain(b1, b2):
    raw = (b1 << 8) | b2
    return -((raw ^ 0xFFFF) + 1) / 10.0 if raw & 0x8000 else raw / 10.0


def decode_response(cmd, resp):
    data_len = resp[5] if len(resp) > 5 else 0
    payload = resp[6:6 + data_len] if len(resp) > 6 else b""
    info = []

    if cmd == 0x18 and len(payload) >= 1:
        info.append(f"EQ band count: {payload[0]}")
    elif cmd == 0x15 and len(resp) >= 14:
        idx, g, f, q, ft = resp[6], parse_gain(resp[7], resp[8]), (resp[9] << 8) | resp[10], ((resp[11] << 8) | resp[12]) / 100.0, resp[13]
        ftypes = {0: "Peak", 1: "LSC", 2: "HSC", 3: "BPF", 4: "LPF", 5: "HPF", 6: "APF"}
        info.append(f"Band {idx}: {f}Hz, {g:+.1f}dB, Q={q:.2f}, type={ftypes.get(ft, hex(ft))}")
    elif cmd == 0x16 and len(payload) >= 1:
        v = payload[0]
        presets = {240: "BYPASS", 0: "Jazz", 1: "Pop", 2: "Rock", 3: "Dance", 4: "R&B", 5: "Classic", 6: "HipHop", 8: "Retro", 9: "sDamp-1", 10: "sDamp-2"}
        name = f"USER {v - 159}" if 160 <= v <= 169 else presets.get(v, f"Unknown({v})")
        info.append(f"Preset: {name} (0x{v:02X})")
    elif cmd == 0x17 and len(resp) >= 8:
        info.append(f"Global gain: {parse_gain(resp[6], resp[7]):+.1f}dB")
    elif cmd == 0x1A and len(payload) >= 1:
        info.append(f"EQ enabled: {bool(payload[0])}")
    elif cmd == 0x30 and len(resp) > 7:
        name = bytes(resp[7:7 + max(0, data_len - 1)]).decode("utf-8", errors="replace").rstrip("\x00")
        info.append(f"Preset name: '{name}'")
    else:
        if payload:
            vals = " ".join(f"0x{b:02X}" for b in payload)
            info.append(f"Payload ({data_len}B): {vals}")
            text = bytes(payload).decode("utf-8", errors="replace").rstrip("\x00")
            if text and all(32 <= ord(c) < 127 for c in text):
                info.append(f"  As text: '{text}'")
            # Try as 16-bit BE values
            if data_len >= 2 and data_len % 2 == 0:
                words = [(payload[i] << 8) | payload[i+1] for i in range(0, data_len, 2)]
                info.append(f"  As u16 BE: {words}")

    return info


def main():
    log_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "logs")
    os.makedirs(log_dir, exist_ok=True)
    log_path = os.path.join(log_dir, f"probe_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log")

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
    print("FiiO K13 R2R — USB HID Feature Probe v3")
    print(f"Log: {log_path}")
    print("=" * 70)

    dev = usb.core.find(idVendor=VENDOR_ID)
    if dev is None:
        print("\nERROR: No FiiO device found.")
        sys.exit(1)

    print(f"\nDevice: {dev.manufacturer} {dev.product}")
    print(f"VID:PID = {dev.idVendor:04X}:{dev.idProduct:04X}")

    try:
        if dev.is_kernel_driver_active(INTERFACE):
            dev.detach_kernel_driver(INTERFACE)
            print(f"Detached kernel driver from interface {INTERFACE}")
    except (usb.core.USBError, NotImplementedError):
        pass

    usb.util.claim_interface(dev, INTERFACE)
    print(f"Claimed interface {INTERFACE}\n")

    # ================================================================
    # PHASE 1: Discover all responding commands
    # ================================================================
    print("-" * 70)
    print("PHASE 1: Scanning all GET commands (0x00-0xFF)")
    print("-" * 70)
    print()

    results = {}  # cmd -> (resp_bytes, payload_bytes)
    io_err = [0]

    for cmd in range(0x100):
        sys.stdout.write(f"\r  Scanning 0x{cmd:02X}/0xFF...")
        sys.stdout.flush()
        try:
            resp = send_get(dev, cmd, io_errors=io_err)
        except DeviceDead as e:
            print(f"\n\n  DEVICE DIED at CMD 0x{cmd:02X}: {e}")
            print("  Aborting Phase 1.\n")
            break

        if resp is not None:
            data_len = resp[5] if len(resp) > 5 else 0
            payload = bytes(resp[6:6 + data_len]) if len(resp) > 6 else b""
            results[cmd] = (bytes(resp), payload)
            label = KNOWN_CMDS.get(cmd, "UNKNOWN")
            print(f"\r  0x{cmd:02X} [{label}] — {hex_str(resp)}")
            for line in decode_response(cmd, resp):
                print(f"    {line}")

        time.sleep(0.05)

    print(f"\r{'':50}")
    print(f"\nResponding commands: {len(results)}")
    cmds_list = ", ".join(f"0x{c:02X}" for c in sorted(results.keys()))
    print(f"  {cmds_list}\n")

    # Categorize
    zero_cmds = {c for c, (r, p) in results.items() if all(b == 0 for b in p)}
    data_cmds = {c for c in results if c not in zero_cmds}

    print(f"Commands with data: {', '.join(f'0x{c:02X}' for c in sorted(data_cmds))}")
    print(f"Commands returning only zeros: {', '.join(f'0x{c:02X}' for c in sorted(zero_cmds))}\n")

    # ================================================================
    # PHASE 2: Probe sub-values on safe/known EQ commands
    # ================================================================
    if not health_check(dev):
        print("Device not responding — skipping remaining phases.\n")
        _cleanup(dev, log_file)
        return

    print("-" * 70)
    print("PHASE 2: Sub-value probe on known EQ commands")
    print("-" * 70)
    print()

    io_err = [0]
    for cmd in sorted(results.keys()):
        if cmd not in SAFE_FOR_SUB_PROBE:
            continue
        label = KNOWN_CMDS.get(cmd, "UNKNOWN")
        print(f"  0x{cmd:02X} [{label}]:")
        seen = set()
        for sub in range(0x10):
            try:
                resp = send_get(dev, cmd, bytes([sub]), io_errors=io_err)
            except DeviceDead:
                print(f"    DEVICE DIED at sub=0x{sub:02X}")
                break
            if resp:
                h = hex_str(resp)
                if h not in seen:
                    seen.add(h)
                    print(f"    [0x{sub:02X}]: {h}")
                    for line in decode_response(cmd, resp):
                        print(f"      {line}")
            time.sleep(0.03)
        if not seen:
            print("    (no responses)")
        print()

    # ================================================================
    # PHASE 3: Careful single-sub probe on unknown commands
    # ================================================================
    unknown_with_data = sorted(data_cmds - SAFE_FOR_SUB_PROBE - set(KNOWN_CMDS.keys()))
    unknown_zero = sorted(zero_cmds - SAFE_FOR_SUB_PROBE - set(KNOWN_CMDS.keys()))
    all_unknowns = unknown_with_data + unknown_zero

    if all_unknowns and health_check(dev):
        print("-" * 70)
        print("PHASE 3: Careful probe on unknown commands (1 sub-value at a time)")
        print("-" * 70)
        print()

        for cmd in all_unknowns:
            if not health_check(dev):
                print("  Device stopped responding — aborting Phase 3.\n")
                break

            print(f"  0x{cmd:02X}:")
            io_err = [0]
            seen = set()

            # Try sub-values 0x00-0x0F, but bail on first I/O error
            for sub in range(0x10):
                try:
                    resp = send_get(dev, cmd, bytes([sub]), io_errors=io_err)
                except DeviceDead:
                    print(f"    CRASHED at sub=0x{sub:02X} — skipping rest of this command")
                    # Try to recover
                    time.sleep(1)
                    if not health_check(dev):
                        print("    Device unrecoverable — aborting Phase 3.\n")
                        break
                    break

                if resp:
                    h = hex_str(resp)
                    if h not in seen:
                        seen.add(h)
                        print(f"    [0x{sub:02X}]: {h}")
                        for line in decode_response(cmd, resp):
                            print(f"      {line}")
                time.sleep(0.05)
            else:
                if not seen:
                    print("    (no responses with sub-values)")
                print()
                continue
            print()

    # ================================================================
    # PHASE 4: Firmware / device info extraction attempts
    # ================================================================
    if health_check(dev):
        print("-" * 70)
        print("PHASE 4: Firmware & device info extraction")
        print("-" * 70)
        print()

        # Common firmware version command patterns across USB HID devices
        fw_attempts = [
            ("GET 0x01 (often firmware ver)", 0x01, b""),
            ("GET 0x02 (often hardware ver)", 0x02, b""),
            ("GET 0x03 (often serial)", 0x03, b""),
            ("GET 0x04 (often model info)", 0x04, b""),
            ("GET 0x07 (2B response)", 0x07, b""),
            ("GET 0x08 (1B response)", 0x08, b""),
            ("GET 0x0B (2B response)", 0x0B, b""),
            ("GET 0x10 (unknown)", 0x10, b""),
            ("GET 0x20 (returned 0x01)", 0x20, b""),
            # Try reading longer data with multi-byte requests
            ("GET 0x01 sub=0x01", 0x01, b"\x01"),
            ("GET 0x02 sub=0x01", 0x02, b"\x01"),
            ("GET 0x03 sub=0x01", 0x03, b"\x01"),
            ("GET 0x07 sub=0x00 0x00", 0x07, b"\x00\x00"),
            ("GET 0x0B sub=0x00 0x00", 0x0B, b"\x00\x00"),
        ]

        for desc, cmd, data in fw_attempts:
            if not health_check(dev):
                print("  Device stopped responding — aborting.\n")
                break

            io_err = [0]
            try:
                resp = send_get(dev, cmd, data, io_errors=io_err)
            except DeviceDead:
                print(f"  {desc}: DEVICE CRASHED — waiting 2s...")
                time.sleep(2)
                continue

            if resp:
                print(f"  {desc}:")
                print(f"    Raw: {hex_str(resp)}")
                for line in decode_response(cmd, resp):
                    print(f"    {line}")
                # Full payload dump
                payload = resp[6:] if len(resp) > 6 else b""
                nonzero = [i for i, b in enumerate(payload) if b != 0]
                if nonzero:
                    last = max(nonzero) + 1
                    print(f"    Full payload ({last}B): {hex_str(payload[:last], limit=64)}")
            else:
                print(f"  {desc}: no response")

            time.sleep(0.1)

        print()

    # ================================================================
    # SUMMARY
    # ================================================================
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print()
    print(f"Total responding GET commands: {len(results)}")
    print()
    for cmd in sorted(results.keys()):
        resp, payload = results[cmd]
        label = KNOWN_CMDS.get(cmd, "???")
        plen = len(payload)
        is_zero = all(b == 0 for b in payload)
        pstr = "all zeros" if is_zero else hex_str(payload, limit=16)
        print(f"  0x{cmd:02X}  {label:20s}  payload={plen}B  {pstr}")
    print()
    print("=" * 70)
    print("Probe complete.")
    print("=" * 70)

    _cleanup(dev, log_file)


def _cleanup(dev, log_file):
    usb.util.release_interface(dev, INTERFACE)
    try:
        dev.attach_kernel_driver(INTERFACE)
    except (usb.core.USBError, NotImplementedError):
        pass
    log_file.close()


if __name__ == "__main__":
    main()
