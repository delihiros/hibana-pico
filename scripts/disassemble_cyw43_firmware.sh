#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_ARG="${1:-firmware/cyw43/w43439A0_7_95_49_00_firmware.bin}"
if [[ "$BIN_ARG" = /* ]]; then
  BIN="$BIN_ARG"
  BIN_FOR_OBJCOPY="$BIN_ARG"
  BIN_DISPLAY="$BIN_ARG"
else
  BIN="$ROOT/$BIN_ARG"
  BIN_FOR_OBJCOPY="$BIN_ARG"
  BIN_DISPLAY="$BIN_ARG"
fi
OUT_DIR="${2:-$ROOT/target/cyw43}"
OUT="$OUT_DIR/w43439A0_7_95_49_00_firmware.thumb.disasm"
OBJ="$OUT_DIR/w43439A0_7_95_49_00_firmware.thumb.o"
HEAD="$ROOT/firmware/cyw43/w43439A0_7_95_49_00_firmware.thumb.disasm.head.txt"

mkdir -p "$OUT_DIR"

LLVM_OBJCOPY=""
LLVM_OBJDUMP=""
if command -v rustc >/dev/null 2>&1; then
  RUST_SYSROOT="$(rustc --print sysroot)"
  RUST_HOST="$(rustc -vV | sed -n 's/^host: //p')"
  if [ -x "$RUST_SYSROOT/lib/rustlib/$RUST_HOST/bin/llvm-objcopy" ]; then
    LLVM_OBJCOPY="$RUST_SYSROOT/lib/rustlib/$RUST_HOST/bin/llvm-objcopy"
  fi
  if [ -x "$RUST_SYSROOT/lib/rustlib/$RUST_HOST/bin/llvm-objdump" ]; then
    LLVM_OBJDUMP="$RUST_SYSROOT/lib/rustlib/$RUST_HOST/bin/llvm-objdump"
  fi
fi

if [ -n "$LLVM_OBJCOPY" ] && [ -n "$LLVM_OBJDUMP" ]; then
  (
    cd "$ROOT"
    "$LLVM_OBJCOPY" \
      -I binary \
      -O elf32-littlearm \
      --rename-section .data=.text,alloc,load,readonly,code \
      "$BIN_FOR_OBJCOPY" \
      "$OBJ"
  )
  "$LLVM_OBJDUMP" \
    --arch-name=thumb \
    --triple=thumbv7em-none-eabi \
    --section=.text \
    --disassemble-all \
    --print-imm-hex \
    --no-show-raw-insn \
    "$OBJ" > "$OUT"
elif command -v arm-none-eabi-objdump >/dev/null 2>&1; then
  arm-none-eabi-objdump -D -b binary -m arm -M force-thumb "$BIN" > "$OUT"
elif command -v objdump >/dev/null 2>&1; then
  objdump -D -b binary -m arm -M force-thumb "$BIN" > "$OUT"
else
  echo "no llvm-objdump, arm-none-eabi-objdump, or objdump found" >&2
  exit 1
fi

{
  echo "# CYW43439 firmware Thumb disassembly excerpt"
  echo "# Source: $BIN_DISPLAY"
  echo "# Full disassembly: $OUT"
  echo
  sed -n '1,220p' "$OUT"
} > "$HEAD"

echo "$OUT"
