# plan.md -- Hibana/Pico Final Plan

This document is normative.

If a concept is not in this document, it is not part of Hibana/Pico.

Hibana/Pico is not designed by adding runtime intelligence.
Hibana/Pico is designed by deleting every runtime decision that hibana
choreography can express.

## 0. Definition

**Hibana/Pico** is a choreographic WASI Preview 1 microkernel swarm for
Raspberry Pi Pico-class boards.

It must run on:

```text
Raspberry Pi Pico      / RP2040
Raspberry Pi Pico W    / RP2040 + CYW43439
Raspberry Pi Pico 2 W  / RP2350 + CYW43439
```

It is:

```text
a no_std / no_alloc WASI P1 syscall-to-choreography runtime
a node-local microkernel capsule
a swarm substrate for remote object routing
```

It is not:

```text
a general Wasm runtime
a POSIX OS
a network stack
a WASI Preview 2 runtime
a bridge layer
a relay layer
```

The only protocol specification source is:

```text
hibana choreography
```

## 1. Hibana-First Law

If a fact can be expressed by hibana, it must be expressed by hibana.

Hibana owns:

```text
global choreography
role projection
legal message order
label legality
branch / route structure
loop continue / break structure
affine control tokens
capability control messages
endpoint decode
localside progression
```

Hibana/Pico must not reimplement these as:

```text
Rust state machines
runtime protocol inference
manual phase flags
transport heuristics
fallback loops
shape-based request dispatch
stringly route selection
fd-number route selection
board-specific protocol branches
```

The allowed localside vocabulary is:

```text
flow().send()
recv()
offer()
decode()
```

If code outside hibana needs to know "what phase is legal next," the design is
wrong.

## 2. Pico Responsibility Law

Hibana/Pico owns only what hibana cannot own:

```text
WASI P1 import trampoline
bounded Wasm execution capacity
GuestLedger fact storage
memory lease table
ChoreoFS object storage
errno mapping
pending syscall token table
resolver readiness facts
MMIO / GPIO / timer / UART
CYW43439 reset / IRQ / gSPI / Wi-Fi byte movement
transport byte framing
firmware measurement gates
```

None of these may become protocol authority.

They are implementation capacity behind hibana-projected roles.

## 3. NodeCapsule

A node is:

```text
NodeCapsule =
  WASI P1 guest
  + Engine
  + GuestLedger
  + Kernel
  + ChoreoFS
  + Resolver
  + TransportPort
```

Each node owns its own:

```text
Wasm memory
fd materialized view
memory lease table
pending syscall table
object store
resolver queue
endpoint state
transport port
```

Swarm nodes never share:

```text
memory
fd tables
leases
pending tokens
endpoint state
kernel state
object stores
```

Remote communication is always:

```text
Kernel_i <-> Kernel_j
```

over hibana messages carried as transport bytes.

## 4. Runtime Shape

A local syscall has this shape:

```text
ordinary WASI P1 app
  -> WASI P1 import trampoline
  -> Engine role
  -> hibana-projected Kernel role
  -> explicit object route
  -> local object / device / resolver fact
  -> typed return / typed reject / ENOSYS / trap
```

A remote object syscall has this shape:

```text
ordinary WASI P1 app on node A
  -> Engine_A
  -> Kernel_A
  -> hibana message over byte transport
  -> Kernel_B
  -> object route on node B
  -> Kernel_B
  -> hibana message over byte transport
  -> Kernel_A
  -> Engine_A
```

No remote node receives authority over local Wasm memory.

Remote data is copied bytes under typed grants, generations, and choreography.

## 5. Authority Law

Runtime progress is legal only when all required facts intersect:

```text
projected phase accepts label
  + payload decodes
  + control-message history materializes fd view
  + fd view names live object identity
  + object generation matches
  + explicit route arm exists
  + memory lease matches when ptr/len is used
  + pending token matches when async completion is used
  + resolver fact exists when readiness is required
  + policy admits operation
  + linked implementation capacity exists
  => progress
```

Any missing fact gives:

```text
typed reject
ENOSYS
trap
drop-by-policy
```

Not authority:

```text
fd number
path string
network address
transport packet
label hint
lane
interrupt
core id
board type
Cargo feature
```

Only hibana choreography decides protocol legality.

Kernel does not decide what is legal.
Kernel only checks whether the facts required by the currently projected hibana
phase are present.

## 6. WASI P1 Law

The guest may be:

```text
Rust std
no_std Rust
C
Zig
TinyGo
handwritten Wasm
```

The choreography does not care.

The Engine sees only:

```text
WASI P1 import -> EngineReq
EngineRet -> WASI result / errno / trap
```

Disabled imports fail closed.

Unsupported imports fail closed.

No import may:

```text
fake success
choose a fallback route
bypass GuestLedger
bypass memory lease checks
bypass projected choreography
```

WASI Preview 2, WIT, Component Model, and P2 sockets/resources/streams are not
part of the runtime.

## 7. Syscall Stream Law

The syscall stream is guarded by hibana.

Example:

```text
MemBorrowRead
  -> MemGrant
  -> WasiFdWrite
  -> WasiFdWriteRet
  -> MemRelease
```

The Engine may emit a syscall request only when the projected Engine role allows
that label.

The Kernel may answer only when the projected Kernel role allows that response.

Bad syscall order is not recovered.

Bad syscall order is rejected.

## 8. Memory Law

Choreography guards memory protocol order.

Leases guard memory authority.

A memory lease contains:

```text
ptr
len
rights
memory generation
lease id
```

Pointer-backed syscalls require a matching lease.

`memory.grow` creates a fence.

After a fence:

```text
old leases reject
old pending memory completions reject
new access must borrow again
```

Choreography does not inspect memory contents.

The lease table authorizes host access to guest memory.

Remote nodes never receive pointers.

## 9. GuestLedger Law

GuestLedger is app-local fact storage only.

It owns:

```text
fd materialized view
memory lease table
pending syscall token table
quota
errno map
optional preopen manifest view
```

It does not own:

```text
protocol order
route choice
filesystem authority
network authority
device authority
transport authority
scheduler policy
retry policy
```

A pending syscall token is linear.

Completion must match:

```text
token id
token generation
syscall kind
fd
fd generation
lease id
lease generation
object generation
expected length / event / tick
```

A stale token is not recoverable.

## 10. ChoreoFS Law

ChoreoFS is a bounded resource identity store.

It is not POSIX.

Authority chain:

```text
path string
  -> selector
  -> manifest entry
  -> object identity
  -> object generation
  -> explicit route arm
  -> fd materialized view
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
TimerDevice
UartDevice
NetworkDatagram
NetworkStream
NetworkListener
RemoteObject
ManagementObject
TelemetryObject
```

Forbidden:

```text
ambient host filesystem passthrough
cwd authority
inode authority
implicit POSIX mutation
hidden socket authority
unbounded path buffers
unbounded iovec buffers
unbounded dirent buffers
```

## 11. Network / FD Law

Network is Kernel object routing.

Network is not WASI P2.
Network is not socket authority.
Network is not transport authority.

WASI P1 apps reach network objects through fd-visible imports:

```text
sock_send     -> fd_write-like NetworkDatagram / NetworkStream route
sock_recv     -> fd_read-like NetworkDatagram / NetworkStream route
sock_shutdown -> fd_close / quiesce route
sock_accept   -> explicit NetworkListener accept route
```

The app sees:

```text
fd
ptr
len
errno
```

The Kernel sees:

```text
fd materialized view
object identity
object generation
route arm
policy
lease
pending token
projected phase
```

The transport sees:

```text
bytes
```

No network operation may progress from fd number alone.

No socket import may invent route authority.

No transport packet may become syscall authority.

## 12. Route Law

Every semantic route is a hibana route.

Route selection must be represented as:

```text
hibana route arm
or hibana affine control token
or hibana-projected control message
```

Route selection must not be represented as:

```text
if fd == ...
if path starts_with ...
if payload looks like ...
if remote address == ...
if ALPN == ...
if lane == ...
if board == ...
```

Remote object routing is:

```text
fd materialized view
  -> object identity
  -> route arm
  -> Kernel_i <-> Kernel_j
```

There is no bridge.

There is no relay.

There is no `is_remote` semantic bypass.

## 13. Resolver Law

Interrupts and readiness are evidence only.

ISR may:

```text
clear hardware flag
capture bounded metadata
enqueue raw readiness
wake executor
return quickly
```

ISR must not:

```text
call Endpoint methods
decode payloads
inspect fd authority
inspect leases
select routes
allocate
block
```

Resolver converts raw readiness into typed facts:

```text
TimerSleepDone
GpioWaitSatisfied
TransportRxReady
TransportTxReady
BudgetExpired
LeaseFenceDue
NodeHealthChanged
CywReady
```

A resolver fact admits progress only when hibana-projected phase is open.

Resolver is not protocol authority.

## 14. Transport Law

Transport carries bytes only.

A swarm frame may contain:

```text
source node
destination node
session id
session generation
lane
label hint
sequence
payload bytes
auth tag if enabled
```

Transport hints are not authority:

```text
lane
label hint
source address
destination address
packet order
retry count
```

Payload authority begins only after endpoint decode.

Valid substrates:

```text
RP2040 SIO FIFO
RP2350 local substrate
CYW43439 Wi-Fi byte transport
QEMU UDP mesh for proof
host queue for tests
```

Substrate is not semantics.

## 15. Raspberry Pi Pico Law

Raspberry Pi Pico is the smallest non-wireless target.

It must support:

```text
RP2040
thumbv6m-none-eabi
no_std
no_alloc
dual-core role execution
SIO FIFO byte movement
GPIO device routes
timer resolver facts
UART debug sink
bounded WASI P1 guest execution
fd_write / poll_oneoff / proc_exit minimal profile
memory lease checks
memory.grow fence
bad syscall order fail-closed path
```

It must not require:

```text
Wi-Fi
CYW43439
RP2350 capacity
heap allocation
host filesystem
ordinary host std capacity
```

## 16. Raspberry Pi Pico W Law

Raspberry Pi Pico W is the minimum physical wireless swarm target.

It must support:

```text
RP2040 + CYW43439
thumbv6m-none-eabi
no_std
no_alloc
dual-core role execution
SIO FIFO local byte movement
GPIO device routes
timer resolver facts
UART debug sink
CYW43439 reset / IRQ / gSPI bring-up
CYW43439 firmware readiness
Wi-Fi byte transport
SwarmFrame exchange
remote object routing through fd materialized views
NetworkDatagram / NetworkStream / NetworkListener object routing
bounded WASI P1 guest execution
memory lease checks
memory.grow fence
bad syscall order fail-closed path
```

It must not require:

```text
RP2350-only capacity
Pico 2 W-only assumptions
host UDP mesh
shared memory between nodes
WASI Preview 2
Component Model
P2 sockets/resources/streams
hidden relay
bridge object
heap allocation
unbounded packet buffers
```

Pico W is stricter than Pico 2 W.

If a wireless design cannot fit Pico W-class RP2040 capacity, it may be a
Pico 2 W extension, but it is not the minimum wireless Hibana/Pico design.

## 17. Raspberry Pi Pico 2 W Law

Raspberry Pi Pico 2 W is the higher-capacity wireless swarm target.

It must support:

```text
RP2350 + CYW43439
thumbv8m.main-none-eabi
no_std
no_alloc
CYW43439 reset / IRQ / gSPI bring-up
CYW43439 firmware readiness
Wi-Fi byte transport
SwarmFrame exchange
remote object routing
network object routing
management object routing
multi-node choreography
```

It may use RP2350 capacity.

It may not change Hibana/Pico semantics.

A behavior that succeeds only because Pico 2 W has more capacity must be named
as Pico 2 W capacity, not core choreography meaning.

## 18. CYW43439 Law

CYW43439 bring-up has a choreography-visible readiness prefix.

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
CywFailed -> no Wi-Fi fallback
```

gSPI byte traffic is not hibana choreography.

It is machine implementation capacity.

Wi-Fi is transport.

Wi-Fi is not route authority.

## 19. Pico Capacity Law

Every Pico firmware path must be bounded.

Bounded resources:

```text
role programs
endpoint slab
tap buffer
transport frame queue
swarm frame payload
fragment buffer
memory lease table
pending syscall table
fd view
ChoreoFS object table
directory entries
path selectors
iovec copies
management image chunks
resolver readiness queue
UART/debug buffer
```

Forbidden on Pico firmware paths:

```text
Vec
Box
Rc
Arc
String as runtime storage
dynamic task spawning
recursive parser
unbounded guest-controlled loops
unbounded import table allocation
unbounded path expansion
unbounded packet reassembly
```

Host-only proof capacity may not define Pico semantics.

Host success is not Pico success.

## 20. Feature Law

Cargo features select implementation capacity only.

Features may:

```text
link implementation bodies
remove implementation bodies
select board substrate
select engine coverage
select syscall handler coverage
select proof/demo artifact embedding
```

Features may not:

```text
change choreography meaning
change route labels
change role ids
make disabled syscalls succeed
introduce fallback semantics
introduce compatibility names
```

Profile is capacity.

Choreography is meaning.

Important profiles:

```text
profile-rp2040-pico-min:
  RP2040 non-wireless minimal WASI P1 device profile

profile-rp2040-picow-swarm-min:
  RP2040 + CYW43439 minimum wireless swarm profile

profile-rp2350-pico2w-swarm-min:
  RP2350 + CYW43439 higher-capacity wireless swarm profile

profile-host-linux-wasip1-full:
  host proof profile for wider ordinary WASI P1 coverage
```

## 21. Spec Generation Law

Repeated protocol facts must have one source.

Generate from single internal specs:

```text
labels
WASI P1 import coverage
handler availability
typed ENOSYS / typed reject disposition
route arm ids
profile capability matrix
projection accessors
coverage tests
```

Forbidden:

```text
manual duplicate label tables
manual duplicate syscall tables
manual duplicate profile truth tables
manual duplicate route ids
```

One meaning gets one source.

## 22. Verification Law

A release-quality tree must prove:

```text
No-P2 surface
No-WIT surface
No-Component-Model surface
No-bridge surface
ordinary wasm32-wasip1 artifact path
import trampoline -> EngineReq path
Engine -> Kernel hibana choreography path
fd materialized view checks
memory lease checks
memory.grow fence
pending syscall token checks
ChoreoFS object authority
NetworkObject routing without P2
RemoteObject routing without bridge
resolver readiness admission
swarm nodes do not share memory
transport label_hint is demux only
endpoint decode owns payload authority
management update requires fence / quiesce / generation
```

Raspberry Pi Pico verification:

```text
thumbv6m-none-eabi build
real RP2040 hardware run
fd_write path
poll_oneoff path
proc_exit path
bad-order fail-closed path
firmware size measurement
SRAM measurement
stack high-water measurement
```

Raspberry Pi Pico W verification:

```text
thumbv6m-none-eabi build
real RP2040 + CYW43439 bring-up
CywReady reached
transport cannot open before CywReady
two physical Pico W nodes exchange SwarmFrame bytes
network object route works through fd
remote object route works through fd
firmware size / SRAM / stack measurements
```

Raspberry Pi Pico 2 W verification:

```text
thumbv8m.main-none-eabi build
real RP2350 + CYW43439 bring-up
two physical Pico 2 W nodes exchange SwarmFrame bytes
three or more nodes run composed choreography
remote object / network object / management object phases pass
firmware size / SRAM / stack measurements
```

QEMU proof is useful.

QEMU proof is not physical success.

## 23. Publication Law

Do not claim Raspberry Pi Pico support until RP2040 physical gates pass.

Do not claim Raspberry Pi Pico W support until RP2040 + CYW43439 physical gates
pass.

Do not claim Raspberry Pi Pico 2 W Wi-Fi swarm completion until RP2350 +
CYW43439 physical swarm gates pass.

Do not claim production management security while using demo authentication.

Demo auth must be named demo auth.

## 24. Non-Goals

```text
WASI Preview 2
WIT
Component Model
P2 sockets/resources/streams
full POSIX filesystem
ambient host filesystem
general-purpose OS scheduler
hard-real-time arbitrary Wasm execution
transport-level semantic routing
bridge object
relay bypass
automatic protocol recovery
heap-required Pico runtime
BLE as swarm backbone
unauthenticated production management
```

## 25. Final Principle

Only hibana choreography decides protocol legality.
Only hibana projection defines local progress.
Only hibana route/control tokens express choices.
Only endpoint decode gives payload meaning.
Only control-message history materializes fd views.
Only leases authorize guest memory access.
Only pending tokens authorize async completion.
Only resolver facts admit readiness.
Only object generations keep resources live.
Only transport carries bytes.
Only bounded static capacity is valid on Pico firmware.

```text
No P2.
No bridge.
No relay.
No hidden fallback.
No heap-required Pico path.
No runtime protocol intelligence outside hibana.
```

Only choreography on real Pico-class hardware.
