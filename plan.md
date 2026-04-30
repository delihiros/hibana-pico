# plan.md — Hibana/Pico Current Final Plan

This document is the current plan, not a historical backlog. Old bring-up
phases, temporary milestones, and compatibility names are intentionally removed.
If a concept is not in this document, it is not part of the design.

## 0. Definition

**Hibana/Pico** is a **choreographic WASI microkernel swarm**.

A single node runs ordinary WASI Preview 1 apps behind session-typed syscall,
memory, device, timer, resolver, and resource protocols.

A swarm is a set of those nodes connected by substrate transport and composed by
hibana choreography.

The final runtime shape is:

```text
ordinary WASI P1 app
  -> WASI P1 import trampoline
  -> Engine role
  -> composed hibana choreography
  -> Kernel role
  -> explicit route arm
  -> local device / ChoreoFS object / remote object / network object
  -> typed return or typed reject
```

There is no independent `bridge` object.
Core0 is not a relay.
Core0 is a Kernel role in the composed choreography.

The specification source is one thing:

```text
choreography
```

## 1. Constitution

Hibana/Pico deliberately excludes these runtime authority sources:

```text
WASI Preview 2
WIT runtime
Component Model loader
WASI 0.3 async semantics
P2 sockets/resources/streams
hidden bridge objects
hidden relay logic
hidden scheduler fallback
hidden network fallback
```

The smallest vocabulary is:

```text
Guest:
  fd, ptr, len, errno

GuestLedger:
  fd materialized view, memory lease, pending syscall token, quota, errno map

Kernel:
  choreography, endpoint, control message, route arm, resolver

Resource:
  ChoreoFS object

Transport:
  bytes only
```

Everything else is implementation capacity behind this vocabulary.

## 2. Authority Law

Runtime progress is legal only when all required typed facts intersect.

```text
projected phase accepts label
  + payload decodes
  + control-message history materializes the fd view
  + fd view points to a ChoreoFS object
  + object kind selects an explicit route arm
  + lease / generation / policy checks pass
  + readiness evidence exists when required
  + linked implementation capacity exists
  => progress

any missing term
  => typed reject / ENOSYS / trap / drop-by-policy
```

The fd view is not authority. It is only a materialized local view of projected
control messages.

ChoreoFS is not authority. It stores resource identity, object generation, and
bounded data. Choreography owns legal progress.

Cargo features are not authority. They only add or remove implementation
capacity.

## 3. Current Implementation State

The current tree has these non-historical implementation claims:

```text
hibana dependency:
  uses hibana crate 0.2.0
  does not edit ../hibana

No-P2:
  no wasm32-wasip2 runtime target
  no WIT runtime
  no Component Model loader
  no P2 socket/resource/stream surface

No-bridge:
  no PicoBridge runtime object
  no fd.is_remote relay path
  no send_packet_to_remote semantic bypass

WASI P1:
  real Rust-built wasm32-wasip1 artifacts are used
  wasm-engine-core is syscall-agnostic
  PicoWasiImportTrampoline owns the WASI P1 import boundary
  WASI P1 import trampoline maps enabled imports into typed EngineReq values
  disabled imports fail closed
  host/full profile admits the 46 Preview 1 imports at the surface

Baker Link / RP2040:
  traffic-light guest is a real WASI P1 artifact
  fd_write drives GPIO through Engine -> Kernel -> GPIO choreography
  poll_oneoff waits through Kernel -> Timer -> resolver choreography
  bad syscall order rejects

Memory:
  pointer-backed syscalls use leases
  memory.grow creates a fence
  stale lease / stale memory generation rejects

ChoreoFS:
  path strings are selectors, not authority
  path_open mints object fds through control-message history
  StaticBlob / ConfigCell / AppendLog / ImageSlot / DirectoryView are bounded
  host-backed storage is a backend for tests, not ambient host authority

Network:
  WASI P1 sock_* imports are ingress only
  sock_send / sock_recv / sock_shutdown normalize into fd/object routes
  sock_accept requires an explicit NetworkListener accept route
  no P2 socket semantics exist

Swarm:
  one choreography connects coordinator/sensor/actuator/gateway flows
  six-process QEMU swarm runner exists
  remote sample, remote actuator, telemetry, network object, and management
    phases are represented as ordinary hibana messages
  nodes do not share memory

CYW43439:
  QEMU model and firmware artifact checks exist
  CYW boot prefix is modeled as typed readiness before transport open
  real Pico 2 W gSPI/Wi-Fi remains the main hardware boundary
```

## 4. No Bypass Path

The main path must not rely on temporary or bypass wiring.

The current critical path is real in these ways:

```text
WASI app:
  built as wasm32-wasip1 artifact
  executed by the core WASI P1 path
  imports fd_write / poll_oneoff / path / sock / proc as Preview 1 imports

Local side:
  uses Endpoint flow().send(), recv(), offer(), decode()
  does not mutate endpoint state by hand

Resolver:
  admits timer / GPIO / transport / budget readiness as facts
  does not become protocol authority

Swarm:
  host/QEMU proofs use separate node state
  payloads are copied bytes, not shared memory
```

Historical bring-up demos may still exist as examples, but they are not the
final proof target. The final proof target is ordinary WASI P1 app bytes wired
through choreography.

## 5. Feature Model

Features are Cargo features only. They select implementation capacity.

Important profile bundles:

```text
profile-rp2040-baker-min:
  platform-rp2040
  machine-sio/timer/gpio/uart
  wasm-engine-core
  wasip1-sys-fd-write
  wasip1-sys-poll-oneoff
  wasip1-sys-proc-exit
  wasip1-ctrl-common
  wasip1-ledger-pico-min

profile-pico2w-swarm-min:
  platform-rp2350
  machine-sio/timer/gpio/uart/cyw43439
  wasm-engine-core
  selected WASI P1 syscall handlers
  wasip1-ctrl-common
  swarm-frame / remote object / datagram object / management capacity

profile-host-linux-wasip1-full:
  platform-host-linux
  wasm-engine-wasip1-full
  wasip1-sys-full
  wasip1-ctrl-common
  wasip1-ledger-host-full
```

Feature law:

```text
features may add or remove implementation bodies
features may not change choreography meaning
features may not change route labels
features may not change role ids
features may not make disabled syscalls succeed silently
```

## 6. WASI P1 Rule

The choreography side is app-agnostic.

It does not care whether the guest app is Rust std, no_std Rust, C, Zig, TinyGo,
or handwritten Wasm.

It only sees the syscall/control message stream emitted by the engine/import
trampoline.

```text
valid app:
  emits syscall/control messages in a legal projected phase
  passes fd view / lease / pending / policy / resolver checks
  progresses

bad app:
  emits syscall/control messages outside the projected phase
  or uses stale fd/lease/pending/generation
  or requests an unlinked/import-disabled syscall
  rejects / traps / ENOSYS
```

The app controls its own internal branch and loop. Hibana/Pico judges only the
boundary stream.

## 7. GuestLedger Rule

GuestLedger is app-local fact storage only.

It owns:

```text
fd materialized view
memory lease table
pending syscall table
quota limits
errno mapping
optional preopen/object manifests
```

It does not own:

```text
protocol order
route choice outside explicit route arms
loop control semantics
retry semantics
transport authority
filesystem authority
network authority
```

Pending syscalls are linear tokens. A completion must match id, fd generation,
lease generation, resource generation, and expected kind.

## 8. ChoreoFS Rule

ChoreoFS is a resource identity store.

```text
path string -> selector
manifest entry -> object identity
object identity -> explicit route arm
control message -> fd materialized view
```

Allowed object kinds:

```text
StaticBlob
ConfigCell
AppendLog
ImageSlot
StateSnapshot
DirectoryView
GpioDevice
NetworkDatagram
NetworkStream
NetworkListener
RemoteObject
```

Forbidden:

```text
ambient host filesystem authority
cwd as authority
inode as authority
implicit POSIX mutation
hidden socket object authority
unbounded path/iovec/dirent buffers
```

## 9. Interrupt / Resolver Rule

Interrupts are readiness evidence only.

ISR may:

```text
clear hardware flag
capture bounded metadata
enqueue raw readiness
wake executor/core
return quickly
```

ISR must not:

```text
call Endpoint methods
decode payloads as semantic authority
inspect fd view
inspect lease table
select routes
allocate
block
```

Resolver converts raw readiness into typed ready facts:

```text
TimerSleepDone
GpioWaitSatisfied
TransportRxReady
TransportTxReady
BudgetExpired
LeaseFenceDue
NodeHealthChanged
```

Endpoint progress remains choreography-owned.

## 10. Network / Sock Rule

Network is not WASI P2. Network is ChoreoFS object routing.

```text
sock_send     -> fd_write-like NetworkDatagram/NetworkStream route
sock_recv     -> fd_read-like NetworkDatagram/NetworkStream route
sock_shutdown -> fd_close/quiesce route
sock_accept   -> explicit NetworkListener accept route that mints an object fd
```

No sock import may invent transport semantics or bypass the fd view, lease,
resolver, policy, or projection checks.

## 11. Swarm Rule

Swarm nodes do not share:

```text
memory
endpoint state
runtime owner
guest ledger
lease table
fd view
```

Swarm nodes communicate through:

```text
SwarmFrame bytes
node id
session generation
lane
label hint as demux hint only
payload decoded by endpoint/projection
```

Wi-Fi is a substrate. It is not route authority.

## 12. CYW43439 Rule

CYW43439 bring-up has a choreography-visible boot prefix, but gSPI byte traffic
is not itself hibana choreography.

Required order:

```text
PowerOn
ResetAssert
ResetRelease
ProbeOk
FirmwareChunk*
FirmwareCommit
ClmNvramApply
CywReady
TransportOpen
```

Failure is fail-closed:

```text
bad image hash -> reject
out-of-order chunk -> reject
ready before commit -> reject
transport before CywReady -> reject
MsgCywFailed -> no Wi-Fi fallback path
```

QEMU should emulate this as far as possible. Real Pico 2 W must load the
precompiled firmware over real gSPI/IRQ/reset wiring.

## 13. Verification Gates

Release-quality local verification requires:

```text
cargo test
cargo test --features profile-host-linux-wasip1-full
bash scripts/check_wasip1_guest_builds.sh
HIBANA_PICO_SKIP_QEMU_SWARM=1 bash scripts/check_plan_pico_gates.sh
```

Release-quality QEMU verification additionally requires a patched QEMU and must
run without the skip:

```text
bash scripts/check_plan_pico_gates.sh
```

The non-skip gate must exercise the six-process Pico 2 W swarm runner.

The current source guards must reject:

```text
WASI P2 / WIT / Component Model runtime surface
bridge / relay runtime surface
old fd-table / compatibility names
old remote-object metric compatibility names
```

## 14. Achieved

These are considered implemented unless future tests regress:

```text
hibana crate 0.2.0 dependency
no local hibana core edits
No-P2 runtime surface guard
No-bridge runtime surface guard
WASI P1 real artifact build gate
Baker Link WASI P1 traffic-light path
Baker Link LED fds minted from ChoreoFS GpioDevice objects
Baker bad-order fail-closed path
timer resolver admission for poll_oneoff
UART device choreography proof
memory lease + memory.grow fence
GuestLedger pending token model
ChoreoFS host/full resource store
ordinary Rust std host/full profile
sock_* as WASI P1 ingress over network objects
remote object route-arm proofs
management/hot-swap lease/fence/quiesce proofs
six-node swarm host proof
Pico firmware builds for RP2040 and RP2350 targets
QEMU CYW43439 model and patched QEMU runner scripts
measurement/budget gates for current firmware images
```

## 15. Remaining Work

These are the real remaining tasks.

### 15.1 QEMU Non-Skip Gate

Run the full plan gate without `HIBANA_PICO_SKIP_QEMU_SWARM=1` on a machine with
the patched QEMU binary.

Exit:

```text
six QEMU Pico 2 W processes run the current minimal kernels
coordinator sees all sensor WASI P1 markers
remote actuator, telemetry, network object, and management phases pass
```

### 15.2 Real Pico 2 W CYW43439 Bring-Up

Implement and validate real hardware gSPI/reset/IRQ firmware loading.

Exit:

```text
real CYW43439 firmware load reaches CywReady
CLM/NVRAM apply succeeds
transport cannot open before CywReady
WiFiRxReady remains readiness evidence only
```

### 15.3 Real Wi-Fi Transport

Replace QEMU UDP mesh with real CYW43439 Wi-Fi transport on Pico 2 W.

Exit:

```text
two physical Pico 2 W nodes exchange SwarmFrame bytes
label_hint remains demux only
endpoint decode owns payload authority
replay/session generation checks remain active
```

### 15.4 Physical Swarm

Run the composed choreography on two nodes, then three or more nodes.

Exit:

```text
Coordinator/Sensor/Actuator/Gateway roles communicate over real Wi-Fi
remote object read/write works through fd
remote node never accesses local Wasm memory
node revoke invalidates materialized remote object views
```

### 15.5 Real Measurement Gates

Measure the physical system.

Exit:

```text
firmware-load time
Wi-Fi RTT
remote object read latency
remote actuator latency
Core0 max blocked time
stack high-water
SRAM high-water
packet loss recovery cost
management transfer time
```

## 16. Publication Readiness

Before public release:

```text
run all local gates
run non-skip QEMU swarm gate
document QEMU requirements
document Baker Link flash/debug workflow
document firmware artifact licensing and local-only files
document real Pico 2 W status honestly
keep README and plan.md aligned
```

Do not claim real Pico 2 W Wi-Fi swarm completion until the physical transport
and physical swarm gates pass.

## 17. Non-Goals

```text
WASI Preview 2
WIT runtime
Component Model loader
P2 socket/resource/stream authority
full POSIX filesystem
ambient host filesystem passthrough
bridge object as runtime layer
relay path outside choreography
ISR-driven endpoint progress
SPI byte stream as hibana protocol
hidden scheduler retry/fallback
hard-real-time arbitrary Wasm execution
BLE as runtime swarm backbone
unauthenticated production management
```

## 18. Final Principle

```text
Only choreography decides protocol legality.
Only control-message history materializes guest-visible fd views.
Only leases authorize guest memory access.
Only resolver facts admit asynchronous readiness.
Only transport carries bytes.
```

No P2 is needed.
No bridge is needed.
No relay is needed.
No hidden fallback is needed.

Only choreography.
