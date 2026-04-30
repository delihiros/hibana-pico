#!/usr/bin/env python3
"""Extract Pico SDK cyw43-driver CYW43439 firmware headers into binary artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path


BYTE_RE = re.compile(r"0x([0-9a-fA-F]{2})")
FW_LEN_RE = re.compile(r"#define\s+CYW43_WIFI_FW_LEN\s+\((\d+)\)")
CLM_LEN_RE = re.compile(r"#define\s+CYW43_CLM_LEN\s+\((\d+)\)")


def fnv1a32(data: bytes) -> int:
    value = 0x811C9DC5
    for byte in data:
        value ^= byte
        value = (value * 0x01000193) & 0xFFFFFFFF
    return value


def read_header(path: Path) -> tuple[bytes, int, int]:
    text = path.read_text(encoding="utf-8")
    fw_len_match = FW_LEN_RE.search(text)
    clm_len_match = CLM_LEN_RE.search(text)
    if fw_len_match is None or clm_len_match is None:
        raise SystemExit(f"{path}: missing CYW43_WIFI_FW_LEN/CYW43_CLM_LEN")

    data = bytes(int(match.group(1), 16) for match in BYTE_RE.finditer(text))
    return data, int(fw_len_match.group(1)), int(clm_len_match.group(1))


def write_bytes(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("header", type=Path)
    parser.add_argument("--out-dir", type=Path, default=Path("firmware/cyw43"))
    parser.add_argument("--pico-sdk-commit", default="unknown")
    parser.add_argument("--cyw43-driver-commit", default="unknown")
    args = parser.parse_args()

    combined, fw_len, clm_len = read_header(args.header)
    clm_offset = ((fw_len + 511) // 512) * 512
    fw = combined[:fw_len]
    clm = combined[clm_offset : clm_offset + clm_len]
    if len(fw) != fw_len:
        raise SystemExit("firmware header shorter than CYW43_WIFI_FW_LEN")
    if len(clm) != clm_len:
        raise SystemExit("firmware header shorter than padded firmware + CYW43_CLM_LEN")

    stem = "w43439A0_7_95_49_00"
    write_bytes(args.out_dir / f"{stem}_combined.bin", combined)
    write_bytes(args.out_dir / f"{stem}_firmware.bin", fw)
    write_bytes(args.out_dir / f"{stem}_clm.bin", clm)

    manifest = {
        "source": {
            "pico_sdk_commit": args.pico_sdk_commit,
            "cyw43_driver_commit": args.cyw43_driver_commit,
            "header_path": str(args.header),
        },
        "artifacts": {
            "combined": {
                "path": f"{stem}_combined.bin",
                "len": len(combined),
                "sha256": sha256(combined),
                "fnv1a32": f"0x{fnv1a32(combined):08x}",
            },
            "firmware": {
                "path": f"{stem}_firmware.bin",
                "len": len(fw),
                "sha256": sha256(fw),
                "fnv1a32": f"0x{fnv1a32(fw):08x}",
            },
            "clm": {
                "path": f"{stem}_clm.bin",
                "len": len(clm),
                "offset_in_combined": clm_offset,
                "sha256": sha256(clm),
                "fnv1a32": f"0x{fnv1a32(clm):08x}",
            },
        },
    }
    (args.out_dir / f"{stem}.manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
