#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_main)]

#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::{
    arch::naked_asm,
    cell::UnsafeCell,
    mem::MaybeUninit,
    ptr::{read_volatile, write_volatile},
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
use hibana::{
    Endpoint, g,
    g::{Msg, Role},
    substrate::{
        AttachError, CpError, SessionKit,
        binding::NoBinding,
        ids::SessionId,
        program::{RoleProgram, project},
        runtime::{Config, CounterClock},
        tap::TapEvent,
    },
};
#[cfg(all(target_arch = "arm", target_os = "none"))]
use hibana_pico::{
    choreography::protocol::{
        EngineLabelUniverse, EngineReq, LABEL_PUBLISH_ALERT, LABEL_PUBLISH_NORMAL,
        LABEL_SAMPLE_REQ, LABEL_YIELD_REQ, LABEL_YIELD_RET, PublishAlertControl,
        PublishNormalControl,
    },
    kernel::engine::wasm::{BAD_ROUTE_EARLY_YIELD_WASM_GUEST, GuestTrap, TinyWasmInstance},
    machine::rp2040::sio::Rp2040SioBackend,
    substrate::exec::{park, run_current_task, signal, wait_until},
    substrate::transport::SioTransport,
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(link_section = ".boot2")]
#[used]
static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" {
    static __stack_top: u32;
    static __core1_stack_top: u32;
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_BASE: usize = 0xD000_0000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_CPUID: *const u32 = SIO_BASE as *const u32;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const UART0_BASE: usize = 0x4003_4000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const UARTDR: *mut u32 = UART0_BASE as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const UARTFR: *const u32 = (UART0_BASE + 0x18) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const UARTIBRD: *mut u32 = (UART0_BASE + 0x24) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const UARTFBRD: *mut u32 = (UART0_BASE + 0x28) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const UARTLCR_H: *mut u32 = (UART0_BASE + 0x2c) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const UARTCR: *mut u32 = (UART0_BASE + 0x30) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const UART_TXFF: u32 = 1 << 5;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESULT_FAILURE: u32 = 0x4849_4641;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SLAB_BYTES: usize = 40 * 1024;

#[cfg(all(target_arch = "arm", target_os = "none"))]
macro_rules! route_program {
    () => {{
        let publish_normal_arm = g::seq(
            g::send::<Role<0>, Role<0>, PublishNormalControl, 0>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_PUBLISH_NORMAL, u32>, 0>(),
        );
        let publish_alert_arm = g::seq(
            g::send::<Role<0>, Role<0>, PublishAlertControl, 0>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_PUBLISH_ALERT, u32>, 0>(),
        );
        g::seq(
            g::send::<Role<1>, Role<0>, Msg<LABEL_SAMPLE_REQ, u32>, 0>(),
            g::seq(
                g::route(publish_normal_arm, publish_alert_arm),
                g::seq(
                    g::send::<Role<1>, Role<0>, Msg<LABEL_YIELD_REQ, ()>, 0>(),
                    g::send::<Role<0>, Role<1>, Msg<LABEL_YIELD_RET, ()>, 0>(),
                ),
            ),
        )
    }};
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
static CORE0_PROGRAM: RoleProgram<0> = project(&route_program!());
#[cfg(all(target_arch = "arm", target_os = "none"))]
static CORE1_PROGRAM: RoleProgram<1> = project(&route_program!());

#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoTransport = SioTransport<Rp2040SioBackend>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoKit = SessionKit<'static, DemoTransport, EngineLabelUniverse, CounterClock, 2>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type SupervisorEndpoint = Endpoint<'static, 0>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type EngineEndpoint = Endpoint<'static, 1>;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_DEMO_RESULT: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
static mut UART_READY: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
static mut RUNTIME_READY: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
static mut UART_LOCK_WANT: [u32; 2] = [0; 2];
#[cfg(all(target_arch = "arm", target_os = "none"))]
static mut UART_LOCK_TURN: u32 = 0;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[repr(C)]
struct VectorTable {
    initial_stack_pointer: *const u32,
    reset: unsafe extern "C" fn() -> !,
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl Sync for VectorTable {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(link_section = ".vector_table.reset_vector")]
#[used]
static VECTOR_TABLE: VectorTable = VectorTable {
    initial_stack_pointer: core::ptr::addr_of!(__stack_top) as *const u32,
    reset,
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
struct SharedRuntime {
    clock: CounterClock,
    tap: [TapEvent; 128],
    slab: [u8; SLAB_BYTES],
    session: MaybeUninit<DemoKit>,
    core0_endpoint: MaybeUninit<SupervisorEndpoint>,
    core1_endpoint: MaybeUninit<EngineEndpoint>,
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
impl SharedRuntime {
    const fn new() -> Self {
        Self {
            clock: CounterClock::new(),
            tap: [TapEvent::zero(); 128],
            slab: [0; SLAB_BYTES],
            session: MaybeUninit::uninit(),
            core0_endpoint: MaybeUninit::uninit(),
            core1_endpoint: MaybeUninit::uninit(),
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
struct SharedRuntimeCell(UnsafeCell<SharedRuntime>);

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl Sync for SharedRuntimeCell {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
static SHARED_RUNTIME: SharedRuntimeCell = SharedRuntimeCell(UnsafeCell::new(SharedRuntime::new()));

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(naked)]
#[unsafe(export_name = "Reset")]
pub unsafe extern "C" fn reset() -> ! {
    naked_asm!(
        "ldr r0, =0xD0000000",
        "ldr r0, [r0]",
        "ldr r2, ={entry}",
        "cmp r0, #0",
        "beq 1f",
        "ldr r1, ={core1_stack_top}",
        "mov sp, r1",
        "bx r2",
        "1:",
        "bx r2",
        core1_stack_top = sym __core1_stack_top,
        entry = sym reset_entry,
    )
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" fn reset_entry() -> ! {
    match core_id() {
        0 => core0_main(),
        _ => core1_main(),
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core_id() -> u32 {
    unsafe { read_volatile(SIO_CPUID) }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn uart_lock() {
    let me = core_id() as usize;
    let other = 1usize.saturating_sub(me);
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(UART_LOCK_WANT[me]), 1);
        write_volatile(core::ptr::addr_of_mut!(UART_LOCK_TURN), other as u32);
    }
    while unsafe { read_volatile(core::ptr::addr_of!(UART_LOCK_WANT[other])) } != 0
        && unsafe { read_volatile(core::ptr::addr_of!(UART_LOCK_TURN)) } == other as u32
    {
        core::hint::spin_loop();
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn uart_unlock() {
    let me = core_id() as usize;
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(UART_LOCK_WANT[me]), 0);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn shared_runtime_ptr() -> *mut SharedRuntime {
    SHARED_RUNTIME.0.get()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe fn shared_supervisor_endpoint() -> &'static mut SupervisorEndpoint {
    unsafe { &mut *(*shared_runtime_ptr()).core0_endpoint.as_mut_ptr() }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe fn shared_engine_endpoint() -> &'static mut EngineEndpoint {
    unsafe { &mut *(*shared_runtime_ptr()).core1_endpoint.as_mut_ptr() }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn uart_init() {
    unsafe {
        write_volatile(UARTCR, 0);
        write_volatile(UARTIBRD, 67);
        write_volatile(UARTFBRD, 52);
        write_volatile(UARTLCR_H, 0x60);
        write_volatile(UARTCR, 0x101);
        UART_READY = 1;
    }
    signal();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn uart_putc(byte: u8) {
    while unsafe { read_volatile(UARTFR) } & UART_TXFF != 0 {}
    unsafe { write_volatile(UARTDR, byte as u32) };
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn uart_puts(text: &str) {
    for byte in text.bytes() {
        if byte == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(byte);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn uart_line(text: &str) {
    uart_lock();
    uart_puts(text);
    uart_puts("\n");
    uart_unlock();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fail_closed(stage: &str) -> ! {
    unsafe {
        HIBANA_DEMO_RESULT = RESULT_FAILURE;
    }
    uart_lock();
    uart_puts(stage);
    uart_puts(" fail\n");
    uart_unlock();
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn attach_or_fail<T>(result: Result<T, AttachError>, stage: &str) -> T {
    match result {
        Ok(value) => value,
        Err(AttachError::Control(CpError::ResourceExhausted)) => {
            uart_lock();
            uart_puts(stage);
            uart_puts(" control resource exhausted\n");
            uart_unlock();
            fail_closed(stage)
        }
        Err(AttachError::Control(_)) => {
            uart_lock();
            uart_puts(stage);
            uart_puts(" control error\n");
            uart_unlock();
            fail_closed(stage)
        }
        Err(AttachError::Rendezvous(_)) => {
            uart_lock();
            uart_puts(stage);
            uart_puts(" rendezvous error\n");
            uart_unlock();
            fail_closed(stage)
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn init_runtime_once() {
    let runtime = shared_runtime_ptr();
    unsafe {
        let session_ptr = (*runtime).session.as_mut_ptr();
        session_ptr.write(SessionKit::new(&(*runtime).clock));
        let kit = &*session_ptr;
        let rv = match kit.add_rendezvous_from_config(
            Config::new(&mut (*runtime).tap, &mut (*runtime).slab)
                .with_universe(EngineLabelUniverse),
            SioTransport::new(Rp2040SioBackend::new()),
        ) {
            Ok(rv) => rv,
            Err(_) => fail_closed("[core0] add rendezvous"),
        };
        let sid = SessionId::new(5);
        (*runtime).core0_endpoint.as_mut_ptr().write(attach_or_fail(
            kit.enter(rv, sid, &CORE0_PROGRAM, NoBinding),
            "[core0] attach endpoint",
        ));
        (*runtime).core1_endpoint.as_mut_ptr().write(attach_or_fail(
            kit.enter(rv, sid, &CORE1_PROGRAM, NoBinding),
            "[core1] attach endpoint",
        ));
        RUNTIME_READY = 1;
    }
    signal();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn supervisor_session(endpoint: &mut SupervisorEndpoint) {
    uart_line("[core0] wait sample");
    match endpoint.recv::<Msg<LABEL_SAMPLE_REQ, u32>>().await {
        Ok(_) => fail_closed("[core0] unexpected sample"),
        Err(_) => fail_closed("[core0] recv sample"),
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_session(endpoint: &mut EngineEndpoint) {
    let mut guest = TinyWasmInstance::new(BAD_ROUTE_EARLY_YIELD_WASM_GUEST)
        .unwrap_or_else(|_| fail_closed("[core1] parse wasm"));
    loop {
        let trap = guest
            .resume()
            .unwrap_or_else(|_| fail_closed("[core1] resume wasm"));
        match trap {
            GuestTrap::HostCall(request) => match request {
                EngineReq::LogU32(_) => fail_closed("[core1] unexpected sample"),
                EngineReq::Wasip1Stdout(_) => fail_closed("[core1] unexpected stdout"),
                EngineReq::Wasip1Stderr(_) => fail_closed("[core1] unexpected stderr"),
                EngineReq::Wasip1Stdin(_) => fail_closed("[core1] unexpected stdin"),
                EngineReq::Wasip1ClockNow => fail_closed("[core1] unexpected clock"),
                EngineReq::Wasip1RandomSeed => fail_closed("[core1] unexpected random"),
                EngineReq::Wasip1Exit(_) => fail_closed("[core1] unexpected exit"),
                EngineReq::TimerSleepUntil(_) => fail_closed("[core1] unexpected timer sleep"),
                EngineReq::GpioSet(_) => fail_closed("[core1] unexpected gpio"),
                EngineReq::Yield => {
                    uart_line("[core1] wasm trap yield");
                    match endpoint.flow::<Msg<LABEL_YIELD_REQ, ()>>() {
                        Ok(_) => fail_closed("[core1] yield phase opened"),
                        Err(_) => {
                            uart_line("[core1] reject early yield");
                            fail_closed("[core1] phase invariant");
                        }
                    }
                }
                _ => fail_closed("unexpected wasi p1 request"),
            },
            GuestTrap::Done => fail_closed("[core1] unexpected done"),
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core0_main() -> ! {
    uart_init();
    uart_line("[core0] hibana wasm route bad");
    uart_line("[core0] init runtime");
    init_runtime_once();
    let endpoint = unsafe { shared_supervisor_endpoint() };
    run_current_task(supervisor_session(endpoint));
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core1_main() -> ! {
    wait_until(|| unsafe { read_volatile(core::ptr::addr_of!(UART_READY)) } != 0);
    wait_until(|| unsafe { read_volatile(core::ptr::addr_of!(RUNTIME_READY)) } != 0);
    let endpoint = unsafe { shared_engine_endpoint() };
    run_current_task(engine_session(endpoint));
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    fail_closed("[panic]")
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}
