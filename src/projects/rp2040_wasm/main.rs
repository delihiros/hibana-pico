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
        ClockNow, EngineLabelUniverse, EngineReq, EngineRet, LABEL_ENGINE_REQ, LABEL_ENGINE_RET,
        LABEL_MEM_BORROW_READ, LABEL_MEM_BORROW_WRITE, LABEL_MEM_COMMIT, LABEL_MEM_FENCE,
        LABEL_MEM_RELEASE, LABEL_MGMT_IMAGE_ACTIVATE, LABEL_MGMT_IMAGE_BEGIN,
        LABEL_MGMT_IMAGE_CHUNK, LABEL_MGMT_IMAGE_END, LABEL_MGMT_IMAGE_STATUS,
        LABEL_WASIP1_CLOCK_NOW, LABEL_WASIP1_CLOCK_NOW_RET, LABEL_WASIP1_EXIT,
        LABEL_WASIP1_RANDOM_SEED, LABEL_WASIP1_RANDOM_SEED_RET, LABEL_WASIP1_STDERR,
        LABEL_WASIP1_STDERR_RET, LABEL_WASIP1_STDIN, LABEL_WASIP1_STDIN_RET, LABEL_WASIP1_STDOUT,
        LABEL_WASIP1_STDOUT_RET, MemBorrow, MemCommit, MemFence, MemFenceReason,
        MemReadGrantControl, MemRelease, MemRights, MemWriteGrantControl, MgmtImageActivate,
        MgmtImageBegin, MgmtImageChunk, MgmtImageEnd, MgmtStatus, MgmtStatusCode, RandomSeed,
        StderrChunk, StdinRequest, StdoutChunk, Wasip1ExitStatus,
    },
    kernel::engine::wasm::{DEMO_WASM_GUEST, GuestTrap, TinyWasmInstance},
    kernel::mgmt::{ActivationBoundary, ImageSlotTable, MgmtControl},
    kernel::swarm::{NodeId, SwarmCredential},
    kernel::wasi::{
        MemoryLeaseTable, WASIP1_CLOCK_DEMO_NANOS, WASIP1_EXIT_DEMO_CODE,
        WASIP1_RANDOM_DEMO_SEED_HI, WASIP1_RANDOM_DEMO_SEED_LO, WASIP1_STDERR_DEMO_TEXT,
        WASIP1_STDIN_DEMO_INPUT, WASIP1_STDIN_DEMO_MAX_LEN, WASIP1_STDOUT_DEMO_TEXT,
    },
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
const ENGINE_LOG_VALUE: u32 = 0x4849_4241;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESULT_SUCCESS: u32 = 0x4849_4f4b;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESULT_FAILURE: u32 = 0x4849_4641;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SLAB_BYTES: usize = 40 * 1024;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_MEMORY_LEN: u32 = 4096;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_MEMORY_EPOCH: u32 = 1;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_STDOUT_PTR: u32 = 1024;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_STDERR_PTR: u32 = 2048;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_STDIN_PTR: u32 = 3072;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_MGMT_MODULE: &[u8] = b"\0asm\x01\0\0\0wasi_snapshot_preview1";
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_MGMT_NODE: NodeId = NodeId::new(1);
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_MGMT_CREDENTIAL: SwarmCredential = SwarmCredential::new(0x4849_4241);
#[cfg(all(target_arch = "arm", target_os = "none"))]
const DEMO_MGMT_SESSION_GENERATION: u16 = 1;

#[cfg(all(target_arch = "arm", target_os = "none"))]
macro_rules! seq_chain {
    ($head:expr, $($tail:expr),+ $(,)?) => {
        g::seq($head, seq_chain!($($tail),+))
    };
    ($last:expr $(,)?) => {
        $last
    };
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
macro_rules! wasm_program {
    () => {
        seq_chain!(
            g::send::<Role<1>, Role<0>, Msg<LABEL_ENGINE_REQ, EngineReq>, 0>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_ENGINE_RET, EngineRet>, 0>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_ENGINE_REQ, EngineReq>, 0>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_ENGINE_RET, EngineRet>, 0>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
            g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDOUT, EngineReq>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
            g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDERR, EngineReq>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDERR_RET, EngineRet>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_WRITE, MemBorrow>, 1>(),
            g::send::<Role<0>, Role<1>, MemWriteGrantControl, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDIN, EngineReq>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDIN_RET, EngineRet>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_COMMIT, MemCommit>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_CLOCK_NOW, EngineReq>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_CLOCK_NOW_RET, EngineRet>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_RANDOM_SEED, EngineReq>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_RANDOM_SEED_RET, EngineRet>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_EXIT, EngineReq>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MGMT_IMAGE_BEGIN, MgmtImageBegin>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MGMT_IMAGE_CHUNK, MgmtImageChunk>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MGMT_IMAGE_END, MgmtImageEnd>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MGMT_IMAGE_ACTIVATE, MgmtImageActivate>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_FENCE, MemFence>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MGMT_IMAGE_ACTIVATE, MgmtImageActivate>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>, 1>(),
        )
    };
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
static CORE0_PROGRAM: RoleProgram<0> = project(&wasm_program!());
#[cfg(all(target_arch = "arm", target_os = "none"))]
static CORE1_PROGRAM: RoleProgram<1> = project(&wasm_program!());

#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoTransport = SioTransport<Rp2040SioBackend>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoKit = SessionKit<'static, DemoTransport, EngineLabelUniverse, CounterClock, 1>;
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
fn uart_bytes(bytes: &[u8]) {
    for &byte in bytes {
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
        let sid = SessionId::new(3);
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
    uart_line("[core0] wait wasm request");
    let request = match endpoint.recv::<Msg<LABEL_ENGINE_REQ, EngineReq>>().await {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv request"),
    };
    match request {
        EngineReq::LogU32(value) => {
            uart_hex_line("[core0] recv log_u32 0x", value);
            if value != ENGINE_LOG_VALUE {
                fail_closed("[core0] log value mismatch");
            }
            let reply = EngineRet::Logged(value);
            match endpoint
                .flow::<Msg<LABEL_ENGINE_RET, EngineRet>>()
                .expect("supervisor flow<ret>")
                .send(&reply)
                .await
            {
                Ok(_) => {}
                Err(_) => fail_closed("[core0] send logged"),
            }
            uart_hex_line("[core0] sent logged 0x", value);
        }
        EngineReq::Yield => fail_closed("[core0] expected log_u32 first"),
        EngineReq::Wasip1Stdout(_) => fail_closed("[core0] expected log_u32 first"),
        EngineReq::Wasip1Stderr(_) => fail_closed("[core0] expected log_u32 first"),
        EngineReq::Wasip1Stdin(_) => fail_closed("[core0] expected log_u32 first"),
        EngineReq::Wasip1ClockNow => fail_closed("[core0] expected log_u32 first"),
        EngineReq::Wasip1RandomSeed => fail_closed("[core0] expected log_u32 first"),
        EngineReq::Wasip1Exit(_) => fail_closed("[core0] expected log_u32 first"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected log_u32 first"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected log_u32 first"),
        _ => fail_closed("unexpected wasi p1 request"),
    }

    uart_line("[core0] wait wasm request");
    let request = match endpoint.recv::<Msg<LABEL_ENGINE_REQ, EngineReq>>().await {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv request"),
    };
    match request {
        EngineReq::Yield => {
            uart_line("[core0] recv yield");
            let reply = EngineRet::Yielded;
            match endpoint
                .flow::<Msg<LABEL_ENGINE_RET, EngineRet>>()
                .expect("supervisor flow<ret>")
                .send(&reply)
                .await
            {
                Ok(_) => {}
                Err(_) => fail_closed("[core0] send yielded"),
            }
            uart_line("[core0] sent yielded");
        }
        EngineReq::LogU32(_) => fail_closed("[core0] expected yield second"),
        EngineReq::Wasip1Stdout(_) => fail_closed("[core0] expected yield second"),
        EngineReq::Wasip1Stderr(_) => fail_closed("[core0] expected yield second"),
        EngineReq::Wasip1Stdin(_) => fail_closed("[core0] expected yield second"),
        EngineReq::Wasip1ClockNow => fail_closed("[core0] expected yield second"),
        EngineReq::Wasip1RandomSeed => fail_closed("[core0] expected yield second"),
        EngineReq::Wasip1Exit(_) => fail_closed("[core0] expected yield second"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected yield second"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected yield second"),
        _ => fail_closed("unexpected wasi p1 request"),
    }

    let mut leases: MemoryLeaseTable<4> = MemoryLeaseTable::new(DEMO_MEMORY_LEN, DEMO_MEMORY_EPOCH);

    uart_line("[core0] wait wasip1 stdout borrow");
    let borrow = match endpoint
        .recv::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
        .await
    {
        Ok(borrow) => borrow,
        Err(_) => fail_closed("[core0] recv stdout mem borrow"),
    };
    if borrow.ptr() != DEMO_STDOUT_PTR
        || borrow.len() as usize != WASIP1_STDOUT_DEMO_TEXT.len()
        || borrow.epoch() != DEMO_MEMORY_EPOCH
    {
        fail_closed("[core0] stdout mem borrow mismatch");
    }
    let grant = leases
        .grant_read(borrow)
        .unwrap_or_else(|_| fail_closed("[core0] grant stdout read lease"));
    if grant.rights() != MemRights::Read {
        fail_closed("[core0] stdout grant rights mismatch");
    }
    match endpoint
        .flow::<MemReadGrantControl>()
        .expect("supervisor flow<stdout mem read grant control>")
        .send(())
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send stdout mem grant"),
    }
    uart_line("[core0] sent stdout mem grant");

    uart_line("[core0] wait wasip1 stdout");
    let request = match endpoint.recv::<Msg<LABEL_WASIP1_STDOUT, EngineReq>>().await {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv wasip1 stdout"),
    };
    let chunk = match request {
        EngineReq::Wasip1Stdout(chunk) => chunk,
        EngineReq::LogU32(_) => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::Yield => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::Wasip1Stderr(_) => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::Wasip1Stdin(_) => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::Wasip1ClockNow => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::Wasip1RandomSeed => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::Wasip1Exit(_) => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected wasip1 stdout"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected wasip1 stdout"),
        _ => fail_closed("unexpected wasi p1 request"),
    };
    leases
        .validate_read_chunk(&chunk)
        .unwrap_or_else(|_| fail_closed("[core0] validate stdout read lease"));
    uart_lock();
    uart_puts("[core0] wasip1 stdout: ");
    uart_bytes(chunk.as_bytes());
    uart_unlock();
    if chunk.as_bytes() != WASIP1_STDOUT_DEMO_TEXT {
        fail_closed("[core0] wasip1 stdout mismatch");
    }
    let reply = EngineRet::Wasip1StdoutWritten(chunk.len() as u8);
    match endpoint
        .flow::<Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>>()
        .expect("supervisor flow<wasip1 stdout ret>")
        .send(&reply)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send wasip1 stdout ret"),
    }
    uart_line("[core0] sent wasip1 stdout ret");
    let release = match endpoint.recv::<Msg<LABEL_MEM_RELEASE, MemRelease>>().await {
        Ok(release) => release,
        Err(_) => fail_closed("[core0] recv stdout mem release"),
    };
    if release.lease_id() != chunk.lease_id() {
        fail_closed("[core0] stdout release lease mismatch");
    }
    leases
        .release(release)
        .unwrap_or_else(|_| fail_closed("[core0] release stdout read lease"));

    uart_line("[core0] wait wasip1 stderr borrow");
    let borrow = match endpoint
        .recv::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
        .await
    {
        Ok(borrow) => borrow,
        Err(_) => fail_closed("[core0] recv stderr mem borrow"),
    };
    if borrow.ptr() != DEMO_STDERR_PTR
        || borrow.len() as usize != WASIP1_STDERR_DEMO_TEXT.len()
        || borrow.epoch() != DEMO_MEMORY_EPOCH
    {
        fail_closed("[core0] stderr mem borrow mismatch");
    }
    let grant = leases
        .grant_read(borrow)
        .unwrap_or_else(|_| fail_closed("[core0] grant stderr read lease"));
    if grant.rights() != MemRights::Read {
        fail_closed("[core0] stderr grant rights mismatch");
    }
    match endpoint
        .flow::<MemReadGrantControl>()
        .expect("supervisor flow<stderr mem read grant control>")
        .send(())
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send stderr mem grant"),
    }
    uart_line("[core0] sent stderr mem grant");

    uart_line("[core0] wait wasip1 stderr");
    let request = match endpoint.recv::<Msg<LABEL_WASIP1_STDERR, EngineReq>>().await {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv wasip1 stderr"),
    };
    let chunk = match request {
        EngineReq::Wasip1Stderr(chunk) => chunk,
        EngineReq::LogU32(_) => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::Yield => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::Wasip1Stdout(_) => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::Wasip1Stdin(_) => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::Wasip1ClockNow => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::Wasip1RandomSeed => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::Wasip1Exit(_) => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected wasip1 stderr"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected wasip1 stderr"),
        _ => fail_closed("unexpected wasi p1 request"),
    };
    leases
        .validate_read_chunk(&chunk)
        .unwrap_or_else(|_| fail_closed("[core0] validate stderr read lease"));
    uart_lock();
    uart_puts("[core0] wasip1 stderr: ");
    uart_bytes(chunk.as_bytes());
    uart_unlock();
    if chunk.as_bytes() != WASIP1_STDERR_DEMO_TEXT {
        fail_closed("[core0] wasip1 stderr mismatch");
    }
    let reply = EngineRet::Wasip1StderrWritten(chunk.len() as u8);
    match endpoint
        .flow::<Msg<LABEL_WASIP1_STDERR_RET, EngineRet>>()
        .expect("supervisor flow<wasip1 stderr ret>")
        .send(&reply)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send wasip1 stderr ret"),
    }
    uart_line("[core0] sent wasip1 stderr ret");
    let release = match endpoint.recv::<Msg<LABEL_MEM_RELEASE, MemRelease>>().await {
        Ok(release) => release,
        Err(_) => fail_closed("[core0] recv stderr mem release"),
    };
    if release.lease_id() != chunk.lease_id() {
        fail_closed("[core0] stderr release lease mismatch");
    }
    leases
        .release(release)
        .unwrap_or_else(|_| fail_closed("[core0] release stderr read lease"));

    uart_line("[core0] wait wasip1 stdin borrow");
    let borrow = match endpoint
        .recv::<Msg<LABEL_MEM_BORROW_WRITE, MemBorrow>>()
        .await
    {
        Ok(borrow) => borrow,
        Err(_) => fail_closed("[core0] recv stdin mem borrow"),
    };
    if borrow.ptr() != DEMO_STDIN_PTR
        || borrow.len() as usize != WASIP1_STDIN_DEMO_MAX_LEN as usize
        || borrow.epoch() != DEMO_MEMORY_EPOCH
    {
        fail_closed("[core0] stdin mem borrow mismatch");
    }
    let grant = leases
        .grant_write(borrow)
        .unwrap_or_else(|_| fail_closed("[core0] grant stdin write lease"));
    if grant.rights() != MemRights::Write {
        fail_closed("[core0] stdin grant rights mismatch");
    }
    match endpoint
        .flow::<MemWriteGrantControl>()
        .expect("supervisor flow<stdin mem write grant control>")
        .send(())
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send stdin mem grant"),
    }
    uart_line("[core0] sent stdin mem grant");

    uart_line("[core0] wait wasip1 stdin");
    let request = match endpoint.recv::<Msg<LABEL_WASIP1_STDIN, EngineReq>>().await {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv wasip1 stdin"),
    };
    let request = match request {
        EngineReq::Wasip1Stdin(request) => request,
        EngineReq::LogU32(_) => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::Yield => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::Wasip1Stdout(_) => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::Wasip1Stderr(_) => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::Wasip1ClockNow => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::Wasip1RandomSeed => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::Wasip1Exit(_) => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected wasip1 stdin"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected wasip1 stdin"),
        _ => fail_closed("unexpected wasi p1 request"),
    };
    leases
        .validate_write_request(&request)
        .unwrap_or_else(|_| fail_closed("[core0] validate stdin write lease request"));
    if (request.max_len() as usize) < WASIP1_STDIN_DEMO_INPUT.len() {
        fail_closed("[core0] wasip1 stdin max mismatch");
    }
    uart_lock();
    uart_puts("[core0] wasip1 stdin: ");
    uart_bytes(WASIP1_STDIN_DEMO_INPUT);
    uart_unlock();
    let chunk = match hibana_pico::choreography::protocol::StdinChunk::new(WASIP1_STDIN_DEMO_INPUT)
    {
        Ok(chunk) => chunk.with_lease(request.lease_id()),
        Err(_) => fail_closed("[core0] make wasip1 stdin chunk"),
    };
    leases
        .validate_write_chunk(&chunk)
        .unwrap_or_else(|_| fail_closed("[core0] validate stdin write lease chunk"));
    let reply = EngineRet::Wasip1StdinRead(chunk);
    match endpoint
        .flow::<Msg<LABEL_WASIP1_STDIN_RET, EngineRet>>()
        .expect("supervisor flow<wasip1 stdin ret>")
        .send(&reply)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send wasip1 stdin ret"),
    }
    uart_line("[core0] sent wasip1 stdin ret");
    let commit = match endpoint.recv::<Msg<LABEL_MEM_COMMIT, MemCommit>>().await {
        Ok(commit) => commit,
        Err(_) => fail_closed("[core0] recv stdin mem commit"),
    };
    if commit.lease_id() != request.lease_id() || commit.written() as usize != chunk.len() {
        fail_closed("[core0] stdin commit mismatch");
    }
    leases
        .commit(commit)
        .unwrap_or_else(|_| fail_closed("[core0] commit stdin write lease"));
    let release = match endpoint.recv::<Msg<LABEL_MEM_RELEASE, MemRelease>>().await {
        Ok(release) => release,
        Err(_) => fail_closed("[core0] recv stdin mem release"),
    };
    if release.lease_id() != request.lease_id() {
        fail_closed("[core0] stdin release lease mismatch");
    }
    leases
        .release(release)
        .unwrap_or_else(|_| fail_closed("[core0] release stdin write lease"));

    uart_line("[core0] wait wasip1 clock");
    let request = match endpoint
        .recv::<Msg<LABEL_WASIP1_CLOCK_NOW, EngineReq>>()
        .await
    {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv wasip1 clock"),
    };
    match request {
        EngineReq::Wasip1ClockNow => {
            uart_line("[core0] wasip1 clock now");
            let reply = EngineRet::Wasip1ClockNow(ClockNow::new(WASIP1_CLOCK_DEMO_NANOS));
            match endpoint
                .flow::<Msg<LABEL_WASIP1_CLOCK_NOW_RET, EngineRet>>()
                .expect("supervisor flow<wasip1 clock ret>")
                .send(&reply)
                .await
            {
                Ok(_) => {}
                Err(_) => fail_closed("[core0] send wasip1 clock ret"),
            }
            uart_line("[core0] sent wasip1 clock ret");
        }
        EngineReq::LogU32(_) => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::Yield => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::Wasip1Stdout(_) => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::Wasip1Stderr(_) => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::Wasip1Stdin(_) => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::Wasip1RandomSeed => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::Wasip1Exit(_) => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected wasip1 clock"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected wasip1 clock"),
        _ => fail_closed("unexpected wasi p1 request"),
    }

    uart_line("[core0] wait wasip1 random");
    let request = match endpoint
        .recv::<Msg<LABEL_WASIP1_RANDOM_SEED, EngineReq>>()
        .await
    {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv wasip1 random"),
    };
    match request {
        EngineReq::Wasip1RandomSeed => {
            uart_line("[core0] wasip1 random seed");
            let reply =
                EngineRet::Wasip1RandomSeed(hibana_pico::choreography::protocol::RandomSeed::new(
                    WASIP1_RANDOM_DEMO_SEED_LO,
                    WASIP1_RANDOM_DEMO_SEED_HI,
                ));
            match endpoint
                .flow::<Msg<LABEL_WASIP1_RANDOM_SEED_RET, EngineRet>>()
                .expect("supervisor flow<wasip1 random ret>")
                .send(&reply)
                .await
            {
                Ok(_) => {}
                Err(_) => fail_closed("[core0] send wasip1 random ret"),
            }
            uart_line("[core0] sent wasip1 random ret");
        }
        EngineReq::LogU32(_) => fail_closed("[core0] expected wasip1 random"),
        EngineReq::Yield => fail_closed("[core0] expected wasip1 random"),
        EngineReq::Wasip1Stdout(_) => fail_closed("[core0] expected wasip1 random"),
        EngineReq::Wasip1Stderr(_) => fail_closed("[core0] expected wasip1 random"),
        EngineReq::Wasip1Stdin(_) => fail_closed("[core0] expected wasip1 random"),
        EngineReq::Wasip1ClockNow => fail_closed("[core0] expected wasip1 random"),
        EngineReq::Wasip1Exit(_) => fail_closed("[core0] expected wasip1 random"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected wasip1 random"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected wasip1 random"),
        _ => fail_closed("unexpected wasi p1 request"),
    }

    uart_line("[core0] wait wasip1 exit");
    let request = match endpoint.recv::<Msg<LABEL_WASIP1_EXIT, EngineReq>>().await {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv wasip1 exit"),
    };
    match request {
        EngineReq::Wasip1Exit(status) => {
            if status.code() != WASIP1_EXIT_DEMO_CODE {
                fail_closed("[core0] wasip1 exit code mismatch");
            }
            uart_line("[core0] wasip1 exit");
        }
        EngineReq::LogU32(_) => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::Yield => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::Wasip1Stdout(_) => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::Wasip1Stderr(_) => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::Wasip1Stdin(_) => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::Wasip1ClockNow => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::Wasip1RandomSeed => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::TimerSleepUntil(_) => fail_closed("[core0] expected wasip1 exit"),
        EngineReq::GpioSet(_) => fail_closed("[core0] expected wasip1 exit"),
        _ => fail_closed("unexpected wasi p1 request"),
    }

    let mut images: ImageSlotTable<2, 64> = ImageSlotTable::new();
    let mut mgmt_leases: MemoryLeaseTable<1> = MemoryLeaseTable::new(DEMO_MEMORY_LEN, 1);
    let mgmt_grant = MgmtControl::install_grant(
        DEMO_MGMT_NODE,
        DEMO_MGMT_CREDENTIAL,
        DEMO_MGMT_SESSION_GENERATION,
        0,
        1,
    );
    mgmt_leases
        .grant_read(MemBorrow::new(0, 8, 1))
        .unwrap_or_else(|_| fail_closed("[core0] seed mgmt lease"));

    uart_line("[core0] wait mgmt image begin");
    let begin = match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_BEGIN, MgmtImageBegin>>()
        .await
    {
        Ok(begin) => begin,
        Err(_) => fail_closed("[core0] recv mgmt begin"),
    };
    let status = images
        .begin_with_control(
            mgmt_grant,
            DEMO_MGMT_NODE,
            DEMO_MGMT_CREDENTIAL,
            DEMO_MGMT_SESSION_GENERATION,
            begin,
        )
        .unwrap_or_else(|_| fail_closed("[core0] mgmt begin image"));
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .expect("supervisor flow<mgmt begin status>")
        .send(&status)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send mgmt begin status"),
    }

    uart_line("[core0] wait mgmt image chunk");
    let chunk = match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_CHUNK, MgmtImageChunk>>()
        .await
    {
        Ok(chunk) => chunk,
        Err(_) => fail_closed("[core0] recv mgmt chunk"),
    };
    let status = images
        .chunk_with_control(
            mgmt_grant,
            DEMO_MGMT_NODE,
            DEMO_MGMT_CREDENTIAL,
            DEMO_MGMT_SESSION_GENERATION,
            chunk,
        )
        .unwrap_or_else(|_| fail_closed("[core0] mgmt image chunk"));
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .expect("supervisor flow<mgmt chunk status>")
        .send(&status)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send mgmt chunk status"),
    }

    uart_line("[core0] wait mgmt image end");
    let end = match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_END, MgmtImageEnd>>()
        .await
    {
        Ok(end) => end,
        Err(_) => fail_closed("[core0] recv mgmt end"),
    };
    let status = images
        .end_with_control(
            mgmt_grant,
            DEMO_MGMT_NODE,
            DEMO_MGMT_CREDENTIAL,
            DEMO_MGMT_SESSION_GENERATION,
            end,
        )
        .unwrap_or_else(|_| fail_closed("[core0] mgmt image end"));
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .expect("supervisor flow<mgmt end status>")
        .send(&status)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send mgmt end status"),
    }

    uart_line("[core0] wait mgmt activate before fence");
    let activate = match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_ACTIVATE, MgmtImageActivate>>()
        .await
    {
        Ok(activate) => activate,
        Err(_) => fail_closed("[core0] recv mgmt activate before fence"),
    };
    let activation_error = match images.activate_with_control(
        mgmt_grant,
        DEMO_MGMT_NODE,
        DEMO_MGMT_CREDENTIAL,
        DEMO_MGMT_SESSION_GENERATION,
        activate,
        ActivationBoundary::single_node(
            !mgmt_leases.has_outstanding_leases(),
            true,
            mgmt_leases.epoch(),
        ),
    ) {
        Ok(_) => fail_closed("[core0] mgmt activate unexpectedly succeeded"),
        Err(error) => error,
    };
    if activation_error != hibana_pico::kernel::mgmt::ImageSlotError::NeedFence {
        fail_closed("[core0] mgmt activate failed for wrong reason");
    }
    let status = activation_error.status(activate.slot());
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .expect("supervisor flow<mgmt need fence status>")
        .send(&status)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send mgmt need fence status"),
    }
    uart_line("[core0] mgmt activate needs fence");

    let fence = match endpoint.recv::<Msg<LABEL_MEM_FENCE, MemFence>>().await {
        Ok(fence) => fence,
        Err(_) => fail_closed("[core0] recv mgmt mem fence"),
    };
    if fence.reason() != MemFenceReason::HotSwap {
        fail_closed("[core0] mgmt fence reason mismatch");
    }
    mgmt_leases.fence(fence);
    uart_line("[core0] mgmt mem fenced");

    uart_line("[core0] wait mgmt activate after fence");
    let activate = match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_ACTIVATE, MgmtImageActivate>>()
        .await
    {
        Ok(activate) => activate,
        Err(_) => fail_closed("[core0] recv mgmt activate after fence"),
    };
    let status = images
        .activate_with_control(
            mgmt_grant,
            DEMO_MGMT_NODE,
            DEMO_MGMT_CREDENTIAL,
            DEMO_MGMT_SESSION_GENERATION,
            activate,
            ActivationBoundary::single_node(
                !mgmt_leases.has_outstanding_leases(),
                true,
                mgmt_leases.epoch(),
            ),
        )
        .unwrap_or_else(|_| fail_closed("[core0] mgmt activate after fence"));
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .expect("supervisor flow<mgmt activate status>")
        .send(&status)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send mgmt activate status"),
    }
    uart_line("[core0] mgmt hotswap ok");
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_session(endpoint: &mut EngineEndpoint) {
    let mut guest = TinyWasmInstance::new(DEMO_WASM_GUEST)
        .unwrap_or_else(|_| fail_closed("[core1] parse wasm"));
    loop {
        let trap = guest
            .resume()
            .unwrap_or_else(|_| fail_closed("[core1] resume wasm"));
        match trap {
            GuestTrap::HostCall(request) => {
                match request {
                    EngineReq::LogU32(value) => {
                        uart_hex_line("[core1] wasm trap log_u32 0x", value)
                    }
                    EngineReq::Yield => uart_line("[core1] wasm trap yield"),
                    EngineReq::Wasip1Stdout(_) => fail_closed("[core1] unexpected stdout"),
                    EngineReq::Wasip1Stderr(_) => fail_closed("[core1] unexpected stderr"),
                    EngineReq::Wasip1Stdin(_) => fail_closed("[core1] unexpected stdin"),
                    EngineReq::Wasip1ClockNow => fail_closed("[core1] unexpected clock"),
                    EngineReq::Wasip1RandomSeed => fail_closed("[core1] unexpected random"),
                    EngineReq::Wasip1Exit(_) => fail_closed("[core1] unexpected exit"),
                    EngineReq::TimerSleepUntil(_) => fail_closed("[core1] unexpected timer sleep"),
                    EngineReq::GpioSet(_) => fail_closed("[core1] unexpected gpio"),
                    _ => fail_closed("unexpected wasi p1 request"),
                }
                match endpoint
                    .flow::<Msg<LABEL_ENGINE_REQ, EngineReq>>()
                    .expect("engine flow<req>")
                    .send(&request)
                    .await
                {
                    Ok(_) => {}
                    Err(_) => fail_closed("[core1] send request"),
                }
                let reply = match endpoint.recv::<Msg<LABEL_ENGINE_RET, EngineRet>>().await {
                    Ok(reply) => reply,
                    Err(_) => fail_closed("[core1] recv reply"),
                };
                match reply {
                    EngineRet::Logged(value) => {
                        uart_hex_line("[core1] wasm reply logged 0x", value)
                    }
                    EngineRet::Yielded => uart_line("[core1] wasm reply yielded"),
                    EngineRet::Wasip1StdoutWritten(_) => {
                        fail_closed("[core1] unexpected stdout reply")
                    }
                    EngineRet::Wasip1StderrWritten(_) => {
                        fail_closed("[core1] unexpected stderr reply")
                    }
                    EngineRet::Wasip1StdinRead(_) => fail_closed("[core1] unexpected stdin reply"),
                    EngineRet::Wasip1ClockNow(_) => fail_closed("[core1] unexpected clock reply"),
                    EngineRet::Wasip1RandomSeed(_) => {
                        fail_closed("[core1] unexpected random reply")
                    }
                    EngineRet::TimerSleepDone(_) => fail_closed("[core1] unexpected timer reply"),
                    EngineRet::GpioSetDone(_) => fail_closed("[core1] unexpected gpio reply"),
                    _ => fail_closed("unexpected wasi p1 reply"),
                }
                guest
                    .complete_host_call(reply)
                    .unwrap_or_else(|_| fail_closed("[core1] complete host call"));
            }
            GuestTrap::Done => break,
        }
    }

    let chunk = StdoutChunk::new(WASIP1_STDOUT_DEMO_TEXT)
        .unwrap_or_else(|_| fail_closed("[core1] make wasip1 stdout"));
    if chunk.as_bytes() != WASIP1_STDOUT_DEMO_TEXT {
        fail_closed("[core1] wasip1 stdout mismatch");
    }
    let borrow = MemBorrow::new(DEMO_STDOUT_PTR, chunk.len() as u8, DEMO_MEMORY_EPOCH);
    match endpoint
        .flow::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
        .expect("engine flow<stdout mem borrow read>")
        .send(&borrow)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send stdout mem borrow"),
    }
    let grant = match endpoint.recv::<MemReadGrantControl>().await {
        Ok(grant) => grant,
        Err(_) => fail_closed("[core1] recv stdout mem grant"),
    };
    let (rights, lease_id) = grant
        .decode_handle()
        .unwrap_or_else(|_| fail_closed("[core1] decode stdout mem grant"));
    if rights != MemRights::Read.tag() || lease_id > u8::MAX as u64 {
        fail_closed("[core1] stdout mem grant mismatch");
    }
    let lease_id = lease_id as u8;
    let chunk = chunk.with_lease(lease_id);
    uart_line("[core1] req wasip1 stdout");
    let request = EngineReq::Wasip1Stdout(chunk);
    match endpoint
        .flow::<Msg<LABEL_WASIP1_STDOUT, EngineReq>>()
        .expect("engine flow<wasip1 stdout>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send wasip1 stdout"),
    }
    let reply = match endpoint
        .recv::<Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv wasip1 stdout ret"),
    };
    match reply {
        EngineRet::Wasip1StdoutWritten(written) => {
            if written != chunk.len() as u8 {
                fail_closed("[core1] wasip1 stdout written mismatch");
            }
            uart_line("[core1] wasip1 stdout written");
        }
        EngineRet::Logged(_) => fail_closed("[core1] expected wasip1 stdout ret"),
        EngineRet::Yielded => fail_closed("[core1] expected wasip1 stdout ret"),
        EngineRet::Wasip1StderrWritten(_) => fail_closed("[core1] expected wasip1 stdout ret"),
        EngineRet::Wasip1StdinRead(_) => fail_closed("[core1] expected wasip1 stdout ret"),
        EngineRet::Wasip1ClockNow(_) => fail_closed("[core1] expected wasip1 stdout ret"),
        EngineRet::Wasip1RandomSeed(_) => fail_closed("[core1] expected wasip1 stdout ret"),
        EngineRet::TimerSleepDone(_) => fail_closed("[core1] expected wasip1 stdout ret"),
        EngineRet::GpioSetDone(_) => fail_closed("[core1] expected wasip1 stdout ret"),
        _ => fail_closed("unexpected wasi p1 reply"),
    }
    let release = MemRelease::new(lease_id);
    match endpoint
        .flow::<Msg<LABEL_MEM_RELEASE, MemRelease>>()
        .expect("engine flow<stdout mem release>")
        .send(&release)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send stdout mem release"),
    }
    uart_line("[core1] stdout mem released");

    let chunk = StderrChunk::new(WASIP1_STDERR_DEMO_TEXT)
        .unwrap_or_else(|_| fail_closed("[core1] make wasip1 stderr"));
    if chunk.as_bytes() != WASIP1_STDERR_DEMO_TEXT {
        fail_closed("[core1] wasip1 stderr mismatch");
    }
    let borrow = MemBorrow::new(DEMO_STDERR_PTR, chunk.len() as u8, DEMO_MEMORY_EPOCH);
    match endpoint
        .flow::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
        .expect("engine flow<stderr mem borrow read>")
        .send(&borrow)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send stderr mem borrow"),
    }
    let grant = match endpoint.recv::<MemReadGrantControl>().await {
        Ok(grant) => grant,
        Err(_) => fail_closed("[core1] recv stderr mem grant"),
    };
    let (rights, lease_id) = grant
        .decode_handle()
        .unwrap_or_else(|_| fail_closed("[core1] decode stderr mem grant"));
    if rights != MemRights::Read.tag() || lease_id > u8::MAX as u64 {
        fail_closed("[core1] stderr mem grant mismatch");
    }
    let lease_id = lease_id as u8;
    let chunk = chunk.with_lease(lease_id);
    uart_line("[core1] req wasip1 stderr");
    let request = EngineReq::Wasip1Stderr(chunk);
    match endpoint
        .flow::<Msg<LABEL_WASIP1_STDERR, EngineReq>>()
        .expect("engine flow<wasip1 stderr>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send wasip1 stderr"),
    }
    let reply = match endpoint
        .recv::<Msg<LABEL_WASIP1_STDERR_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv wasip1 stderr ret"),
    };
    match reply {
        EngineRet::Wasip1StderrWritten(written) => {
            if written != chunk.len() as u8 {
                fail_closed("[core1] wasip1 stderr written mismatch");
            }
            uart_line("[core1] wasip1 stderr written");
        }
        EngineRet::Logged(_) => fail_closed("[core1] expected wasip1 stderr ret"),
        EngineRet::Yielded => fail_closed("[core1] expected wasip1 stderr ret"),
        EngineRet::Wasip1StdoutWritten(_) => fail_closed("[core1] expected wasip1 stderr ret"),
        EngineRet::Wasip1StdinRead(_) => fail_closed("[core1] expected wasip1 stderr ret"),
        EngineRet::Wasip1ClockNow(_) => fail_closed("[core1] expected wasip1 stderr ret"),
        EngineRet::Wasip1RandomSeed(_) => fail_closed("[core1] expected wasip1 stderr ret"),
        EngineRet::TimerSleepDone(_) => fail_closed("[core1] expected wasip1 stderr ret"),
        EngineRet::GpioSetDone(_) => fail_closed("[core1] expected wasip1 stderr ret"),
        _ => fail_closed("unexpected wasi p1 reply"),
    }
    let release = MemRelease::new(lease_id);
    match endpoint
        .flow::<Msg<LABEL_MEM_RELEASE, MemRelease>>()
        .expect("engine flow<stderr mem release>")
        .send(&release)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send stderr mem release"),
    }
    uart_line("[core1] stderr mem released");

    let request = StdinRequest::new(WASIP1_STDIN_DEMO_MAX_LEN)
        .unwrap_or_else(|_| fail_closed("[core1] make wasip1 stdin request"));
    if (request.max_len() as usize) < WASIP1_STDIN_DEMO_INPUT.len() {
        fail_closed("[core1] wasip1 stdin max mismatch");
    }
    let borrow = MemBorrow::new(DEMO_STDIN_PTR, request.max_len(), DEMO_MEMORY_EPOCH);
    match endpoint
        .flow::<Msg<LABEL_MEM_BORROW_WRITE, MemBorrow>>()
        .expect("engine flow<stdin mem borrow write>")
        .send(&borrow)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send stdin mem borrow"),
    }
    let grant = match endpoint.recv::<MemWriteGrantControl>().await {
        Ok(grant) => grant,
        Err(_) => fail_closed("[core1] recv stdin mem grant"),
    };
    let (rights, lease_id) = grant
        .decode_handle()
        .unwrap_or_else(|_| fail_closed("[core1] decode stdin mem grant"));
    if rights != MemRights::Write.tag() || lease_id > u8::MAX as u64 {
        fail_closed("[core1] stdin mem grant mismatch");
    }
    let lease_id = lease_id as u8;
    let request = hibana_pico::choreography::protocol::StdinRequest::new_with_lease(
        lease_id,
        request.max_len(),
    )
    .unwrap_or_else(|_| fail_closed("[core1] make leased stdin request"));
    uart_line("[core1] req wasip1 stdin");
    let request = EngineReq::Wasip1Stdin(request);
    match endpoint
        .flow::<Msg<LABEL_WASIP1_STDIN, EngineReq>>()
        .expect("engine flow<wasip1 stdin>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send wasip1 stdin"),
    }
    let reply = match endpoint
        .recv::<Msg<LABEL_WASIP1_STDIN_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv wasip1 stdin ret"),
    };
    let chunk = match reply {
        EngineRet::Wasip1StdinRead(chunk) => chunk,
        EngineRet::Logged(_) => fail_closed("[core1] expected wasip1 stdin ret"),
        EngineRet::Yielded => fail_closed("[core1] expected wasip1 stdin ret"),
        EngineRet::Wasip1StdoutWritten(_) => fail_closed("[core1] expected wasip1 stdin ret"),
        EngineRet::Wasip1StderrWritten(_) => fail_closed("[core1] expected wasip1 stdin ret"),
        EngineRet::Wasip1ClockNow(_) => fail_closed("[core1] expected wasip1 stdin ret"),
        EngineRet::Wasip1RandomSeed(_) => fail_closed("[core1] expected wasip1 stdin ret"),
        EngineRet::TimerSleepDone(_) => fail_closed("[core1] expected wasip1 stdin ret"),
        EngineRet::GpioSetDone(_) => fail_closed("[core1] expected wasip1 stdin ret"),
        _ => fail_closed("unexpected wasi p1 reply"),
    };
    if chunk.lease_id() != lease_id || chunk.as_bytes() != WASIP1_STDIN_DEMO_INPUT {
        fail_closed("[core1] wasip1 stdin mismatch");
    }
    uart_line("[core1] wasip1 stdin read");
    let commit = MemCommit::new(lease_id, chunk.len() as u8);
    match endpoint
        .flow::<Msg<LABEL_MEM_COMMIT, MemCommit>>()
        .expect("engine flow<stdin mem commit>")
        .send(&commit)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send stdin mem commit"),
    }
    let release = MemRelease::new(lease_id);
    match endpoint
        .flow::<Msg<LABEL_MEM_RELEASE, MemRelease>>()
        .expect("engine flow<stdin mem release>")
        .send(&release)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send stdin mem release"),
    }
    uart_line("[core1] stdin mem released");

    let expected = ClockNow::new(WASIP1_CLOCK_DEMO_NANOS);
    if expected.nanos() != WASIP1_CLOCK_DEMO_NANOS {
        fail_closed("[core1] wasip1 clock expected mismatch");
    }
    uart_line("[core1] req wasip1 clock");
    let request = EngineReq::Wasip1ClockNow;
    match endpoint
        .flow::<Msg<LABEL_WASIP1_CLOCK_NOW, EngineReq>>()
        .expect("engine flow<wasip1 clock>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send wasip1 clock"),
    }
    let reply = match endpoint
        .recv::<Msg<LABEL_WASIP1_CLOCK_NOW_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv wasip1 clock ret"),
    };
    match reply {
        EngineRet::Wasip1ClockNow(now) => {
            if now != expected {
                fail_closed("[core1] wasip1 clock mismatch");
            }
            uart_line("[core1] wasip1 clock now");
        }
        EngineRet::Logged(_) => fail_closed("[core1] expected wasip1 clock ret"),
        EngineRet::Yielded => fail_closed("[core1] expected wasip1 clock ret"),
        EngineRet::Wasip1StdoutWritten(_) => fail_closed("[core1] expected wasip1 clock ret"),
        EngineRet::Wasip1StderrWritten(_) => fail_closed("[core1] expected wasip1 clock ret"),
        EngineRet::Wasip1StdinRead(_) => fail_closed("[core1] expected wasip1 clock ret"),
        EngineRet::Wasip1RandomSeed(_) => fail_closed("[core1] expected wasip1 clock ret"),
        EngineRet::TimerSleepDone(_) => fail_closed("[core1] expected wasip1 clock ret"),
        EngineRet::GpioSetDone(_) => fail_closed("[core1] expected wasip1 clock ret"),
        _ => fail_closed("unexpected wasi p1 reply"),
    }

    let expected = RandomSeed::new(WASIP1_RANDOM_DEMO_SEED_LO, WASIP1_RANDOM_DEMO_SEED_HI);
    if expected.lo() != WASIP1_RANDOM_DEMO_SEED_LO || expected.hi() != WASIP1_RANDOM_DEMO_SEED_HI {
        fail_closed("[core1] wasip1 random expected mismatch");
    }
    uart_line("[core1] req wasip1 random");
    let request = EngineReq::Wasip1RandomSeed;
    match endpoint
        .flow::<Msg<LABEL_WASIP1_RANDOM_SEED, EngineReq>>()
        .expect("engine flow<wasip1 random>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send wasip1 random"),
    }
    let reply = match endpoint
        .recv::<Msg<LABEL_WASIP1_RANDOM_SEED_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv wasip1 random ret"),
    };
    match reply {
        EngineRet::Wasip1RandomSeed(seed) => {
            if seed != expected {
                fail_closed("[core1] wasip1 random mismatch");
            }
            uart_line("[core1] wasip1 random seed");
        }
        EngineRet::Logged(_) => fail_closed("[core1] expected wasip1 random ret"),
        EngineRet::Yielded => fail_closed("[core1] expected wasip1 random ret"),
        EngineRet::Wasip1StdoutWritten(_) => fail_closed("[core1] expected wasip1 random ret"),
        EngineRet::Wasip1StderrWritten(_) => fail_closed("[core1] expected wasip1 random ret"),
        EngineRet::Wasip1StdinRead(_) => fail_closed("[core1] expected wasip1 random ret"),
        EngineRet::Wasip1ClockNow(_) => fail_closed("[core1] expected wasip1 random ret"),
        EngineRet::TimerSleepDone(_) => fail_closed("[core1] expected wasip1 random ret"),
        EngineRet::GpioSetDone(_) => fail_closed("[core1] expected wasip1 random ret"),
        _ => fail_closed("unexpected wasi p1 reply"),
    }

    let status = Wasip1ExitStatus::new(WASIP1_EXIT_DEMO_CODE);
    if status.code() != WASIP1_EXIT_DEMO_CODE {
        fail_closed("[core1] wasip1 exit expected mismatch");
    }
    uart_line("[core1] req wasip1 exit");
    let request = EngineReq::Wasip1Exit(status);
    match endpoint
        .flow::<Msg<LABEL_WASIP1_EXIT, EngineReq>>()
        .expect("engine flow<wasip1 exit>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send wasip1 exit"),
    }
    uart_line("[core1] wasip1 exit sent");

    let begin = MgmtImageBegin::new(0, DEMO_MGMT_MODULE.len() as u32, 1);
    uart_line("[core1] mgmt image begin");
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_BEGIN, MgmtImageBegin>>()
        .expect("engine flow<mgmt begin>")
        .send(&begin)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send mgmt begin"),
    }
    match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .await
    {
        Ok(status) if status.code() == MgmtStatusCode::Ok => {}
        Ok(_) => fail_closed("[core1] mgmt begin status mismatch"),
        Err(_) => fail_closed("[core1] recv mgmt begin status"),
    }

    let chunk = MgmtImageChunk::new(0, 0, DEMO_MGMT_MODULE)
        .unwrap_or_else(|_| fail_closed("[core1] make mgmt chunk"));
    uart_line("[core1] mgmt image chunk");
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_CHUNK, MgmtImageChunk>>()
        .expect("engine flow<mgmt chunk>")
        .send(&chunk)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send mgmt chunk"),
    }
    match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .await
    {
        Ok(status) if status.code() == MgmtStatusCode::Ok => {}
        Ok(_) => fail_closed("[core1] mgmt chunk status mismatch"),
        Err(_) => fail_closed("[core1] recv mgmt chunk status"),
    }

    let end = MgmtImageEnd::new(0, DEMO_MGMT_MODULE.len() as u32);
    uart_line("[core1] mgmt image end");
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_END, MgmtImageEnd>>()
        .expect("engine flow<mgmt end>")
        .send(&end)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send mgmt end"),
    }
    match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .await
    {
        Ok(status) if status.code() == MgmtStatusCode::Ok => {}
        Ok(_) => fail_closed("[core1] mgmt end status mismatch"),
        Err(_) => fail_closed("[core1] recv mgmt end status"),
    }

    let activate = MgmtImageActivate::new(0, 2);
    uart_line("[core1] mgmt activate before fence");
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_ACTIVATE, MgmtImageActivate>>()
        .expect("engine flow<mgmt activate before fence>")
        .send(&activate)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send mgmt activate before fence"),
    }
    match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .await
    {
        Ok(status) if status.code() == MgmtStatusCode::NeedFence => {}
        Ok(_) => fail_closed("[core1] mgmt need fence status mismatch"),
        Err(_) => fail_closed("[core1] recv mgmt need fence status"),
    }

    let fence = MemFence::new(MemFenceReason::HotSwap, 2);
    uart_line("[core1] mgmt mem fence");
    match endpoint
        .flow::<Msg<LABEL_MEM_FENCE, MemFence>>()
        .expect("engine flow<mgmt mem fence>")
        .send(&fence)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send mgmt mem fence"),
    }

    uart_line("[core1] mgmt activate after fence");
    match endpoint
        .flow::<Msg<LABEL_MGMT_IMAGE_ACTIVATE, MgmtImageActivate>>()
        .expect("engine flow<mgmt activate after fence>")
        .send(&activate)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send mgmt activate after fence"),
    }
    match endpoint
        .recv::<Msg<LABEL_MGMT_IMAGE_STATUS, MgmtStatus>>()
        .await
    {
        Ok(status) if status.code() == MgmtStatusCode::Ok => {}
        Ok(_) => fail_closed("[core1] mgmt activate status mismatch"),
        Err(_) => fail_closed("[core1] recv mgmt activate status"),
    }
    uart_line("[core1] mgmt hotswap ok");

    unsafe {
        HIBANA_DEMO_RESULT = RESULT_SUCCESS;
    }
    uart_line("[core1] hibana wasm + wasip1 services ok");
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core0_main() -> ! {
    uart_init();
    uart_line("[core0] hibana wasm guest");
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
