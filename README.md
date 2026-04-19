# hibana-pico

`hibana-pico` is a downstream proof that one `hibana` choreography can span both RP2040 cores without changing the `hibana` surface.

The demo is intentionally narrow. Core 1 sends `PING`, Core 0 receives it and sends `PONG`, and both cores print the value they handled over UART. The important point is not the payload itself. The important point is that the application logic stays in plain localside form:

```rust
let ping = endpoint.recv::<Msg<LABEL_PING, u8>>().await?;
endpoint.flow::<Msg<LABEL_PONG, u8>>()?.send(&PONG_VALUE).await?;
```

Board-specific code is confined to the downstream transport/backend and boot glue. `hibana` itself does not gain Pico-specific API or vocabulary.

## What This Proves

- One frozen choreography can be projected into two roles and attached across two RP2040 cores.
- The visible application model remains `hibana::g` plus localside `flow().send()` / `recv()`.
- A board-local substrate can provide async wakeups through RP2040 SIO FIFO and still let the session logic read like normal localside code.
- `hibana` core does not need a Pico-only rescue path for this first proof.

This is a strong technical proof of `hibana`'s shape. It is not yet a finished OS or a full product demo. What it shows well is that session-typed choreography survives contact with a tiny dual-core MCU without collapsing into ad hoc transport code.

## Scope

Current scope is only the minimal dual-core proof:

- `Role 1` on Core 1 sends `PING`
- `Role 0` on Core 0 receives `PING`
- `Role 0` on Core 0 sends `PONG`
- `Role 1` on Core 1 receives `PONG`
- both cores print the handled value

Out of scope for this crate revision:

- Wasm
- syscall routing families
- driver stacks beyond the minimal UART/SIO board glue
- raw SIO smoke binaries that bypass `hibana`

## Layout

- `src/backend.rs`: host queue backend and RP2040 SIO backend
- `src/transport.rs`: `hibana::substrate::Transport` implementation for the board-local FIFO path
- `src/exec.rs`: tiny no-std poll/park/signal glue used by the demo and host parity tests
- `src/bin/hibana-pico-demo.rs`: the actual dual-core RP2040 demo
- `tests/host_sio_ping_pong.rs`: host parity proof of the same localside ping-pong sequence
- `qemu/overlay/`: RP2040 `.c` / `.h` source files copied into an upstream QEMU checkout
- `qemu/patches/`: patches for existing upstream QEMU files only

## Build And Test

From this directory:

```bash
rustup target add thumbv6m-none-eabi
cargo test
cargo build --target thumbv6m-none-eabi --release --bin hibana-pico-demo
```

The host test proves the choreography with `HostQueueBackend`. The RP2040 build proves the same session shape still compiles for `thumbv6m-none-eabi`.

## QEMU Patch Base

The QEMU support under `qemu/overlay/` and `qemu/patches/` was prepared against upstream QEMU commit:

```text
da6c4fe60fee30dd77267764d55b38af9cb89d4b
```

If you choose a different upstream base, patch adjustment may be required.

## Build Patched QEMU

One straightforward flow:

```bash
cd ..
git clone https://github.com/qemu/qemu.git qemu-rp2040
cd qemu-rp2040
git checkout da6c4fe60fee30dd77267764d55b38af9cb89d4b
../hibana-pico/qemu/apply-patches.sh "$PWD"
mkdir build
cd build
../configure --target-list=arm-softmmu
make -j"$(nproc)"
```

QEMU build prerequisites such as `python3`, `meson`, and `ninja` must already be available in `PATH`.

The apply script does two things:

- copies the RP2040 source/header overlay from `qemu/overlay/`
- applies the remaining diffs from `qemu/patches/`

The resulting tree adds:

- RP2040 SoC modeling
- `raspberrypi-pico` machine support
- RP2040 SIO FIFO support
- an `armv7m_set_event()` hook so one core can wake the other out of `wfe`

## Run The Demo

From `hibana-pico/` after the QEMU build finishes:

```bash
timeout 8s ../qemu-rp2040/build/qemu-system-arm \
  -M raspberrypi-pico \
  -kernel target/thumbv6m-none-eabi/release/hibana-pico-demo \
  -nographic \
  -serial mon:stdio
```

Typical output looks like this:

```text
[core0] hibana sio ping-pong
[core0] init runtime
[core0] wait ping
[core1] send ping 0x0000002a
[core0] recv ping 0x0000002a
[core0] sent pong 0x00000055
[core1] recv pong 0x00000055
[core1] hibana sio ping-pong ok
```

Line order between cores can vary slightly because the two CPUs race, but both handled values must appear and the final status must be `ok`.

## QEMU Assets

Patch order is recorded in `qemu/patches/series`.

- `0001-armv7m-set-event-and-rp2040-sio.patch`
- `0002-rp2040-soc-and-raspberrypi-pico-machine.patch`

The RP2040 implementation files themselves live under `qemu/overlay/` as normal `.c` and `.h` sources.
Only existing upstream files are kept as `git apply` patches.
