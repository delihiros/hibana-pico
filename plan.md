# hibana-pico Wasm Sensor Plan

#decision-tree #investigation/embedded #status/active

## Decision Tree (v1)

- current path: [[plan#A. Can hibana-pico keep `hibana` core frozen while growing toward user-authored Wasm apps?|A]] -> [[plan#A1. Does the current downstream proof already preserve the intended `hibana` surface?|A1]] -> [[plan#B. Should the next milestone be a minimal Wasm guest bridge before sensor-node features?|B]] -> [[plan#B1. Should the first Wasm milestone target full WASI and SDK features?|B1]] -> [[plan#C. Should Wi-Fi and Pico W be in the first proof milestone?|C]] -> [[plan#C1. Is the ZeroOS-style sensor-node still the right first real demo after the Wasm bridge?|C1]] -> [[plan#D. Is there enough evidence to choose a specific Wasm engine and budget today?|D]]; selected: [[plan#^branch_d_no|D=no]] <--- [[plan#^branch_current|current]]

### A. Can hibana-pico keep `hibana` core frozen while growing toward user-authored Wasm apps?
- [ ] no ^branch_a_no
- [x] yes; why: the current proof already spans both RP2040 cores without changing the visible `hibana` model; evidence: [[plan#E2. current hibana-pico scope keeps the `hibana` surface intact|E2]], [[plan#E3. one choreography already projects into both RP2040 roles|E3]] ^branch_a_yes

#### A1. Does the current downstream proof already preserve the intended `hibana` surface?
- [ ] no ^branch_a1_no
- [x] yes; why: both README and demo code keep the application path at `hibana::g` plus `recv()` and `flow().send()`; evidence: [[plan#E2. current hibana-pico scope keeps the `hibana` surface intact|E2]], [[plan#E3. one choreography already projects into both RP2040 roles|E3]] ^branch_a1_yes
- next: replace `PING/PONG` with `EngineReq/EngineRet` while keeping the same localside shape, then update [[plan#B. Should the next milestone be a minimal Wasm guest bridge before sensor-node features?|B]] [[plan#^branch_next|next]]

### B. Should the next milestone be a minimal Wasm guest bridge before sensor-node features?
- [ ] no ^branch_b_no
- [x] yes; why: the original Pico vision already defines `Core 0 = supervisor/drivers`, `Core 1 = Wasm engine`, while the current crate still leaves Wasm out of scope; evidence: [[plan#E1. original Pico vision is already Core0 supervisor plus Core1 Wasm engine|E1]], [[plan#E2. current hibana-pico scope keeps the `hibana` surface intact|E2]] ^branch_b_yes

#### B1. Should the first Wasm milestone target full WASI and SDK features?
- [x] no; why: the original roadmap separates foundation, runtime integration, and later ecosystem work, so the first proof only needs a narrow syscall bridge; evidence: [[plan#E1. original Pico vision is already Core0 supervisor plus Core1 Wasm engine|E1]] ^branch_b1_no
- [ ] yes ^branch_b1_yes
- next: define one tiny guest ABI such as `log_u32` plus `yield`, then update [[plan#C. Should Wi-Fi and Pico W be in the first proof milestone?|C]] [[plan#^branch_next|next]]

### C. Should Wi-Fi and Pico W be in the first proof milestone?
- [x] no; why: the sensor-node scenario is a valuable end demo, but the first unresolved gap is still the Wasm guest bridge on top of the existing dual-core choreography proof; evidence: [[plan#E1. original Pico vision is already Core0 supervisor plus Core1 Wasm engine|E1]], [[plan#E4. ZeroOS sensor node is dual-core and sensor-first but still uses globals and polling|E4]] ^branch_c_no
- [ ] yes ^branch_c_yes

#### C1. Is the ZeroOS-style sensor-node still the right first real demo after the Wasm bridge?
- [ ] no ^branch_c1_no
- [x] yes; why: the scenario already gives a concrete dual-core split and concrete devices, but it should be rewritten from global-variable polling into typed choreography roles; evidence: [[plan#E4. ZeroOS sensor node is dual-core and sensor-first but still uses globals and polling|E4]] ^branch_c1_yes
- next: model `Engine`, `Supervisor`, `LightSensor`, `EnvSensor`, and `Publisher`, then update [[plan#D. Is there enough evidence to choose a specific Wasm engine and budget today?|D]] [[plan#^branch_next|next]]

### D. Is there enough evidence to choose a specific Wasm engine and budget today?
- [x] no; why: there is not yet a stable RP2040 size baseline or an engine comparison, and the local cargo/rustup invocation is not yet settled; evidence: [[plan#E5. current toolchain observation blocks a clean RP2040 size baseline|E5]] ^branch_d_no
- [ ] yes ^branch_d_yes
- next: stabilize one toolchain invocation, record the baseline `hibana-pico-demo` footprint, then compare one tiny guest ABI against one engine candidate before updating [[plan#D. Is there enough evidence to choose a specific Wasm engine and budget today?|D]] [[plan#^branch_next|next]]

## Evidence

### E1. original Pico vision is already Core0 supervisor plus Core1 Wasm engine
- source: `/Users/ovm/Code/hibanaworks/ref/ideas/plan_pico.md`
```text
- **System Layer (Kernel)**: Written in Rust + Hibana. Handles IRQs, DMA, and Timing with **mathematical safety** (Choreography). It never crashes.
- **Application Layer (User)**: Written in any WASI-compliant language (Rust, TinyGo, Zig, AssemblyScript). Runs in a Sandbox (Wasm).
- **Interface**: WASI System Calls are translated into **Hibana Messages**.
| **Kernel** | **Hibana Core** | **Core 0** | "The Supervisor". Drives HW, manages resources, schedules Core 1. |
| **Engine** | **Wasm Runtime** | **Core 1** | "The Worker". Executes Wasm instructions, traps syscalls to Kernel. |
| **Devices** | **Drivers** | **Core 0** | Stateless drivers for UART, GPIO, Network. |
```

### E2. current hibana-pico scope keeps the `hibana` surface intact
- source: `/Users/ovm/Code/hibanaworks/hibana-pico/README.md`
```text
`hibana-pico` is a downstream proof that one `hibana` choreography can span both RP2040 cores without changing the `hibana` surface.
The important point is that the application logic stays in plain localside form:
let ping = endpoint.recv::<Msg<LABEL_PING, u8>>().await?;
endpoint.flow::<Msg<LABEL_PONG, u8>>()?.send(&PONG_VALUE).await?;
Board-specific code is confined to the downstream transport/backend and boot glue.
Out of scope for this crate revision:
- Wasm
- syscall routing families
```

### E3. one choreography already projects into both RP2040 roles
- source: `/Users/ovm/Code/hibanaworks/hibana-pico/src/bin/hibana-pico-demo.rs`
```text
const PROGRAM: g::Program<ProgramSteps> = g::seq(
    g::send::<Role<1>, Role<0>, Msg<LABEL_PING, u8>, 0>(),
    g::send::<Role<0>, Role<1>, Msg<LABEL_PONG, u8>, 0>(),
);
static CORE0_PROGRAM: RoleProgram<'static, 0> = project(&PROGRAM);
static CORE1_PROGRAM: RoleProgram<'static, 1> = project(&PROGRAM);
type DemoCore0Endpoint = Endpoint<'static, 0, DemoKit>;
type DemoCore1Endpoint = Endpoint<'static, 1, DemoKit>;
```

### E4. ZeroOS sensor node is dual-core and sensor-first but still uses globals and polling
- source: `/tmp/hibana_zeroos_107_111.txt`
```text
▶CPUコア0：Wi-Fi通信制御
▶CPUコア1：センサ制御
　応答性などの時間制約は厳しくありませんので，各
データはグローバル変数としポーリングで処理します
　CPUコア1のTry Kernelのアプリケーション・プ
ログラムは，センサごとに制御タスクを設けます．
　CPUコア0のアプリケーション・プログラムは，一
定の周期でグローバル変数のセンサのデータをWi-Fi
でサーバに送信します．
```

### E5. current toolchain observation blocks a clean RP2040 size baseline
- source: `/tmp/hibana_pico_toolchain_observation.txt`
```text
$ which cargo
/opt/homebrew/bin/cargo

$ ~/.cargo/bin/rustup target list --installed | rg thumbv6m-none-eabi
thumbv6m-none-eabi

$ ~/.cargo/bin/cargo build --target thumbv6m-none-eabi --release --bin hibana-pico-demo
error[E0463]: can't find crate for `core`
  = note: the `thumbv6m-none-eabi` target may not be installed
```

## Current Understanding
- strongest selected state is [[plan#^branch_d_no|D=no]]: `hibana-pico` should stay downstream-only, insert a minimal Wasm guest bridge before sensor-node features, and postpone Wi-Fi/Pico W until after the bridge works on the existing dual-core choreography proof ^branch_current

## Next Branch
- stabilize one cargo/rustup invocation and record the baseline RP2040 footprint of `hibana-pico-demo`, then replace `PING/PONG` with a narrow `EngineReq/EngineRet` slice before evaluating one concrete Wasm engine candidate for [[plan#D. Is there enough evidence to choose a specific Wasm engine and budget today?|D]] ^branch_next

## Missing Evidence
- run `cd /Users/ovm/Code/hibanaworks/hibana-pico && ~/.cargo/bin/cargo build --target thumbv6m-none-eabi --release --bin hibana-pico-demo` successfully and record `flash/.bss/.data` for [[plan#D. Is there enough evidence to choose a specific Wasm engine and budget today?|D]] ^branch_missing
- compare one no-std Wasm engine candidate against the baseline in `/Users/ovm/Code/hibanaworks/hibana-pico/Cargo.toml`
- prototype the first `EngineReq/EngineRet` localside rewrite in `/Users/ovm/Code/hibanaworks/hibana-pico/src/bin/hibana-pico-demo.rs`
