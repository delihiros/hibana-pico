#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

pattern="${1:-traffic}"
target="thumbv6m-none-eabi"
bin_name="hibana-pico-baker-led-demo"
features="profile-rp2040-baker-min embed-wasip1-artifacts"
expected_result="48494f4b"
expected_stage=""

case "$pattern" in
  traffic)
    ;;
  chaser)
    features="$features baker-chaser-demo"
    ;;
  bad-order)
    features="$features baker-bad-order-demo"
    expected_result="4849524a"
    expected_stage="48490043"
    ;;
  invalid-fd)
    features="$features baker-invalid-fd-demo"
    expected_result="4849524a"
    expected_stage="48490044"
    ;;
  bad-payload)
    features="$features baker-bad-payload-demo"
    expected_result="4849524a"
    expected_stage="48490045"
    ;;
  *)
    echo "usage: $0 {traffic|chaser|bad-order|invalid-fd|bad-payload}" >&2
    exit 2
    ;;
esac

bash ./scripts/check_wasip1_guest_builds.sh
cargo build \
  --target "$target" \
  --release \
  --bin "$bin_name" \
  --features "$features"

elf="target/$target/release/$bin_name"

probe-rs download \
  --chip RP2040 \
  --non-interactive \
  --verify \
  --disable-progressbars \
  "$elf"
probe-rs reset --chip RP2040 --non-interactive

sysroot="$(rustc --print sysroot)"
host="$(rustc -vV | sed -n 's/^host: //p')"
llvm_nm="$sysroot/lib/rustlib/$host/bin/llvm-nm"
if [[ ! -x "$llvm_nm" ]]; then
  echo "missing llvm-nm at $llvm_nm" >&2
  exit 1
fi

symbol_addr() {
  local symbol="$1"
  local value
  value="$("$llvm_nm" -n "$elf" | awk -v sym="$symbol" '$NF == sym { print $1; exit }')"
  if [[ -z "$value" ]]; then
    echo "missing symbol $symbol in $elf" >&2
    exit 1
  fi
  printf '0x%s\n' "$value"
}

read_word() {
  local addr="$1"
  probe-rs read --chip RP2040 --non-interactive b32 "$addr" 1 \
    | awk 'NF { value=$NF } END { print tolower(value) }'
}

result_addr="$(symbol_addr HIBANA_DEMO_RESULT)"
stage_addr="$(symbol_addr HIBANA_DEMO_FAILURE_STAGE)"
result=""
stage=""
deadline=$((SECONDS + ${HIBANA_BAKER_TIMEOUT_SECONDS:-45}))
while :; do
  result="$(read_word "$result_addr")"
  stage="$(read_word "$stage_addr")"
  if [[ "$result" == "$expected_result" ]]; then
    break
  fi
  if [[ "$result" == "48494641" ]]; then
    break
  fi
  if (( SECONDS >= deadline )); then
    break
  fi
  sleep "${HIBANA_BAKER_POLL_SECONDS:-1}"
done

printf 'pattern=%s\n' "$pattern"
printf 'features=%s\n' "$features"
printf 'result_addr=%s result=0x%s expected=0x%s\n' "$result_addr" "$result" "$expected_result"
printf 'stage_addr=%s stage=0x%s\n' "$stage_addr" "$stage"

if [[ "$result" != "$expected_result" ]]; then
  echo "Baker hardware pattern $pattern failed: result mismatch" >&2
  exit 1
fi

if [[ -n "$expected_stage" && "$stage" != "$expected_stage" ]]; then
  echo "Baker hardware pattern $pattern failed: failure-stage mismatch" >&2
  exit 1
fi

echo "Baker hardware pattern $pattern ok"
