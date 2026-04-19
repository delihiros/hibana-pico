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
    g::advanced::steps::{SendStep, SeqSteps, StepCons, StepNil},
    g::advanced::{RoleProgram, project},
    g::{Msg, Role},
    substrate::{AttachError, CpError, SessionId, SessionKit},
    substrate::{
        binding::NoBinding,
        runtime::{Config, CounterClock, LabelUniverse},
        tap::TapEvent,
    },
};
#[cfg(all(target_arch = "arm", target_os = "none"))]
use hibana_pico::{
    backend::Rp2040SioBackend,
    exec::{drive, park, signal, wait_until},
    transport::SioTransport,
};

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
const LABEL_PING: u8 = 1;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const LABEL_PONG: u8 = 2;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PING_VALUE: u8 = 0x2a;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PONG_VALUE: u8 = 0x55;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESULT_SUCCESS: u32 = 0x4849_4f4b;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESULT_FAILURE: u32 = 0x4849_4641;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SLAB_BYTES: usize = 147_456;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[derive(Clone, Copy, Debug, Default)]
struct PingPongLabelUniverse;

#[cfg(all(target_arch = "arm", target_os = "none"))]
impl LabelUniverse for PingPongLabelUniverse {
    const MAX_LABEL: u8 = LABEL_PONG;
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
type PingStep = StepCons<SendStep<Role<1>, Role<0>, Msg<LABEL_PING, u8>, 0>, StepNil>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type PongStep = StepCons<SendStep<Role<0>, Role<1>, Msg<LABEL_PONG, u8>, 0>, StepNil>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type ProgramSteps = SeqSteps<PingStep, PongStep>;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const PROGRAM: g::Program<ProgramSteps> = g::seq(
    g::send::<Role<1>, Role<0>, Msg<LABEL_PING, u8>, 0>(),
    g::send::<Role<0>, Role<1>, Msg<LABEL_PONG, u8>, 0>(),
);

#[cfg(all(target_arch = "arm", target_os = "none"))]
static CORE0_PROGRAM: RoleProgram<'static, 0> = project(&PROGRAM);
#[cfg(all(target_arch = "arm", target_os = "none"))]
static CORE1_PROGRAM: RoleProgram<'static, 1> = project(&PROGRAM);

#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoTransport = SioTransport<Rp2040SioBackend>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoKit = SessionKit<'static, DemoTransport, PingPongLabelUniverse, CounterClock, 1>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoCore0Endpoint = Endpoint<'static, 0, DemoKit>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoCore1Endpoint = Endpoint<'static, 1, DemoKit>;

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
    core0_endpoint: MaybeUninit<DemoCore0Endpoint>,
    core1_endpoint: MaybeUninit<DemoCore1Endpoint>,
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
unsafe fn shared_core0_endpoint() -> &'static mut DemoCore0Endpoint {
    unsafe { &mut *(*shared_runtime_ptr()).core0_endpoint.as_mut_ptr() }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe fn shared_core1_endpoint() -> &'static mut DemoCore1Endpoint {
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
fn uart_hex(value: u32) {
    for shift in (0..8).rev() {
        let nibble = ((value >> (shift * 4)) & 0xf) as u8;
        let ch = match nibble {
            0..=9 => b'0' + nibble,
            _ => b'a' + (nibble - 10),
        };
        uart_putc(ch);
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
fn uart_hex_line(prefix: &str, value: u32) {
    uart_lock();
    uart_puts(prefix);
    uart_hex(value);
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
fn must_attach<T>(result: Result<T, AttachError>, stage: &str) -> T {
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
                .with_universe(PingPongLabelUniverse),
            SioTransport::new(Rp2040SioBackend::new()),
        ) {
            Ok(rv) => rv,
            Err(_) => fail_closed("[core0] add rendezvous"),
        };
        let sid = SessionId::new(1);
        (*runtime).core0_endpoint.as_mut_ptr().write(must_attach(
            kit.enter(rv, sid, &CORE0_PROGRAM, NoBinding),
            "[core0] attach endpoint",
        ));
        (*runtime).core1_endpoint.as_mut_ptr().write(must_attach(
            kit.enter(rv, sid, &CORE1_PROGRAM, NoBinding),
            "[core1] attach endpoint",
        ));
        RUNTIME_READY = 1;
    }
    signal();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn core0_session(endpoint: &mut DemoCore0Endpoint) {
    uart_line("[core0] wait ping");
    let ping = match endpoint.recv::<Msg<LABEL_PING, u8>>().await {
        Ok(ping) => ping,
        Err(_) => fail_closed("[core0] recv ping"),
    };
    uart_hex_line("[core0] recv ping 0x", ping as u32);
    if ping != PING_VALUE {
        fail_closed("[core0] ping mismatch");
    }

    match endpoint
        .flow::<Msg<LABEL_PONG, u8>>()
        .expect("core0 flow<pong>")
        .send(&PONG_VALUE)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send pong"),
    }
    uart_hex_line("[core0] sent pong 0x", PONG_VALUE as u32);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn core1_session(endpoint: &mut DemoCore1Endpoint) {
    uart_hex_line("[core1] send ping 0x", PING_VALUE as u32);
    match endpoint
        .flow::<Msg<LABEL_PING, u8>>()
        .expect("core1 flow<ping>")
        .send(&PING_VALUE)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send ping"),
    }

    let pong = match endpoint.recv::<Msg<LABEL_PONG, u8>>().await {
        Ok(pong) => pong,
        Err(_) => fail_closed("[core1] recv pong"),
    };
    uart_hex_line("[core1] recv pong 0x", pong as u32);

    unsafe {
        HIBANA_DEMO_RESULT = if pong == PONG_VALUE {
            RESULT_SUCCESS
        } else {
            RESULT_FAILURE
        };
    }

    if pong == PONG_VALUE {
        uart_line("[core1] hibana sio ping-pong ok");
    } else {
        uart_line("[core1] hibana sio ping-pong fail");
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core0_main() -> ! {
    uart_init();
    uart_line("[core0] hibana sio ping-pong");
    uart_line("[core0] init runtime");
    init_runtime_once();
    let endpoint = unsafe { shared_core0_endpoint() };
    drive(core0_session(endpoint));
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core1_main() -> ! {
    wait_until(|| unsafe { read_volatile(core::ptr::addr_of!(UART_READY)) } != 0);
    wait_until(|| unsafe { read_volatile(core::ptr::addr_of!(RUNTIME_READY)) } != 0);
    let endpoint = unsafe { shared_core1_endpoint() };
    drive(core1_session(endpoint));
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    fail_closed("[panic]")
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}
