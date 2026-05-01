#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_main)]

#[cfg(any(
    all(
        feature = "baker-bad-order-demo",
        any(
            feature = "baker-chaser-demo",
            feature = "baker-ordinary-std-demo",
            feature = "baker-invalid-fd-demo",
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-chaser-demo",
        any(
            feature = "baker-ordinary-std-demo",
            feature = "baker-invalid-fd-demo",
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-ordinary-std-demo",
        any(
            feature = "baker-invalid-fd-demo",
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-invalid-fd-demo",
        any(
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-bad-payload-demo",
        any(
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-choreofs-demo",
        any(
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-choreofs-bad-path-demo",
        any(
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )
))]
compile_error!("select at most one Baker WASI guest pattern");

#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::{
    arch::{asm, naked_asm},
    cell::UnsafeCell,
    mem::MaybeUninit,
    ptr::{read_volatile, write_volatile},
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
use hibana::{
    Endpoint,
    g::Msg,
    substrate::{
        AttachError, CpError, SessionKit,
        binding::NoBinding,
        ids::SessionId,
        policy::ResolverRef,
        runtime::{Config, CounterClock},
        tap::TapEvent,
    },
};
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    not(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    ))
))]
use hibana_pico::choreography::local::baker_led_blink_roles;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )
))]
use hibana_pico::choreography::local::baker_led_choreofs_blink_roles;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    not(feature = "baker-bad-order-demo")
))]
use hibana_pico::choreography::protocol::PollOneoff;
#[cfg(all(target_arch = "arm", target_os = "none"))]
use hibana_pico::{
    choreography::protocol::{
        BakerTrafficLoopBreakControl, BakerTrafficLoopContinueControl, BudgetRun, BudgetRunMsg,
        EngineLabelUniverse, EngineReq, EngineRet, FdWrite, FdWriteDone, GpioSet, LABEL_GPIO_SET,
        LABEL_GPIO_SET_DONE, LABEL_MEM_BORROW_READ, LABEL_MEM_RELEASE, LABEL_TIMER_SLEEP_DONE,
        LABEL_TIMER_SLEEP_UNTIL, LABEL_WASI_FD_WRITE, LABEL_WASI_FD_WRITE_RET,
        LABEL_WASI_PATH_OPEN, LABEL_WASI_PATH_OPEN_RET, LABEL_WASI_POLL_ONEOFF,
        LABEL_WASI_POLL_ONEOFF_RET, LABEL_WASI_PROC_EXIT, MemBorrow, MemReadGrantControl,
        MemRelease, MemRights, POLICY_BAKER_TRAFFIC_LOOP, PathOpen, PathOpened, PollReady,
        ProcExitStatus, TimerSleepDone, TimerSleepUntil, WASIP1_STREAM_CHUNK_CAPACITY,
    },
    kernel::{
        choreofs::ChoreoFsError,
        engine::wasm::{
            CoreWasip1Instance, CoreWasip1PathCall, CoreWasip1PathKind, CoreWasip1Trap, WasmError,
        },
        fd_object::{GpioFdWriteError, check_gpio_object_fd_write},
        features::Wasip1HandlerSet,
        guest_ledger::GuestLedger,
        resolver::{
            BakerTrafficLoopResolver, InterruptEvent, PicoInterruptResolver, ResolvedInterrupt,
        },
    },
    machine::rp2040::baker_link::{
        BAKER_LINK_CHOREOFS_PREOPEN_FD, BAKER_LINK_LED_ACTIVE_HIGH, BAKER_LINK_LED_PINS,
        BAKER_LINK_TRAFFIC_LIGHT_PATTERN_STEPS, BakerLinkLedResourceStore,
        apply_baker_link_led_bank_set, baker_link_choreofs_ledger, baker_link_led_fd_write_route,
        baker_link_led_resource_store, open_baker_link_choreofs_path,
    },
    machine::rp2040::sio::Rp2040SioBackend,
    machine::rp2040::{clock, timer, uart},
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
    static __data_load_start: u8;
    static mut __data_start: u8;
    static mut __data_end: u8;
    static mut __bss_start: u8;
    static mut __bss_end: u8;
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_BASE: usize = 0xD000_0000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_CPUID: *const u32 = SIO_BASE as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_FIFO_ST: *const u32 = (SIO_BASE + 0x50) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_FIFO_ST_WRITE: *mut u32 = (SIO_BASE + 0x50) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_FIFO_WR: *mut u32 = (SIO_BASE + 0x54) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_FIFO_RD: *const u32 = (SIO_BASE + 0x58) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const FIFO_VLD: u32 = 1 << 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const FIFO_RDY: u32 = 1 << 1;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const FIFO_WOF: u32 = 1 << 2;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const FIFO_ROE: u32 = 1 << 3;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const IO_BANK0_BASE: usize = 0x4001_4000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PADS_BANK0_BASE: usize = 0x4001_c000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_BASE: usize = 0x4000_c000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_RESET_CLR: *mut u32 = (RESETS_BASE + 0x3000) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_IO_BANK0: u32 = 1 << 5;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_PADS_BANK0: u32 = 1 << 8;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OUT_SET: *mut u32 = (SIO_BASE + 0x14) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x18) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OE_SET: *mut u32 = (SIO_BASE + 0x24) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_FUNC_SIO: u32 = 5;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_PAD_DEFAULT: u32 = 0x56;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESULT_SUCCESS: u32 = 0x4849_4f4b;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    any(
        feature = "baker-bad-order-demo",
        feature = "baker-invalid-fd-demo",
        feature = "baker-bad-payload-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )
))]
const RESULT_EXPECTED_REJECT: u32 = 0x4849_524a;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESULT_FAILURE: u32 = 0x4849_4641;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_CORE0_START: u32 = 0x4849_0001;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_CORE1_LAUNCHED: u32 = 0x4849_0002;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_GPIO_READY: u32 = 0x4849_0003;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_RUNTIME_BEGIN: u32 = 0x4849_0004;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_RENDEZVOUS_READY: u32 = 0x4849_0005;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_PROGRAM_READY: u32 = 0x4849_0006;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_ATTACHED: u32 = 0x4849_0007;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_ATTACHED: u32 = 0x4849_0008;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_GPIO_ATTACHED: u32 = 0x4849_0009;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_RUNTIME_READY: u32 = 0x4849_000a;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_FIRST_LED_WRITE_DONE: u32 = 0x4849_000b;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_POLL_ON_DONE: u32 = 0x4849_000c;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_FINAL_LED_WRITE_DONE: u32 = 0x4849_000d;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_POLL_RECV: u32 = 0x4849_0010;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_TIMER_SLEEP_SENT: u32 = 0x4849_0011;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_TIMER_SLEEP_RECV: u32 = 0x4849_0012;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_TIMER_ALARM_ARMED: u32 = 0x4849_0013;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_TIMER_RAW_READY: u32 = 0x4849_0014;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_TIMER_DONE_SENT: u32 = 0x4849_0015;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_FD_WRITE_BEGIN: u32 = 0x4849_0020;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_FD_WRITE_BORROW_RECV: u32 = 0x4849_0021;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_FD_WRITE_GRANT_SENT: u32 = 0x4849_0022;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_FD_WRITE_REQ_RECV: u32 = 0x4849_0023;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_FD_WRITE_GPIO_DONE: u32 = 0x4849_0024;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_PROC_EXIT_RECV: u32 = 0x4849_0025;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_RUN_SEND_BEGIN: u32 = 0x4849_0026;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_RUN_SEND_DONE: u32 = 0x4849_0027;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_BEGIN: u32 = 0x4849_0030;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_FD_WRITE_BEGIN: u32 = 0x4849_0031;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_FD_WRITE_BORROW_SENT: u32 = 0x4849_0032;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RUNTIME_READY_SEEN: u32 = 0x4849_0033;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_ENDPOINT_READY: u32 = 0x4849_0034;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PARSE_DONE: u32 = 0x4849_0035;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PROC_EXIT_SENT: u32 = 0x4849_0036;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_LOOP_CONTINUE_SENT: u32 = 0x4849_0037;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_LOOP_BREAK_SENT: u32 = 0x4849_0038;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_FD_WRITE: u32 = 0x4849_0039;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_POLL_ONEOFF: u32 = 0x4849_003a;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_ENVIRON_SIZES: u32 = 0x4849_003b;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_ENVIRON_GET: u32 = 0x4849_003c;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_ARGS_SIZES: u32 = 0x4849_003d;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_ARGS_GET: u32 = 0x4849_003e;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_MEMORY_GROW: u32 = 0x4849_003f;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_UNSUPPORTED: u32 = 0x4849_0040;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RUN_RECV_BEGIN: u32 = 0x4849_0041;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_BORROW_SEND_ERR: u32 = 0x4849_0042;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RUN_RECV_DONE: u32 = 0x4849_0046;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_RUN_SEND_ERR: u32 = 0x4849_0047;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RUN_RECV_ERR: u32 = 0x4849_0048;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RUN_MISMATCH: u32 = 0x4849_0049;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_RUN_FLOW_ERR: u32 = 0x4849_004a;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_TRUNCATED: u32 = 0x4849_0101;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_INVALID: u32 = 0x4849_0102;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_UNSUPPORTED: u32 = 0x4849_0103;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_STACK_OVERFLOW: u32 = 0x4849_0104;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_STACK_UNDERFLOW: u32 = 0x4849_0105;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_PENDING: u32 = 0x4849_0106;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_REPLY: u32 = 0x4849_0107;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_TRAP: u32 = 0x4849_0108;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_FUEL: u32 = 0x4849_0109;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_RESUME_ERR_OPCODE_BASE: u32 = 0x4849_0200;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "baker-bad-order-demo"
))]
const STAGE_BAD_ORDER_POLL_REJECTED: u32 = 0x4849_0043;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "baker-invalid-fd-demo"
))]
const STAGE_INVALID_FD_REJECTED: u32 = 0x4849_0044;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "baker-bad-payload-demo"
))]
const STAGE_BAD_PAYLOAD_REJECTED: u32 = 0x4849_0045;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "baker-choreofs-bad-path-demo"
))]
const STAGE_BAD_PATH_REJECTED: u32 = 0x4849_004b;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "baker-choreofs-bad-payload-demo"
))]
const STAGE_CHOREOFS_BAD_PAYLOAD_REJECTED: u32 = 0x4849_004c;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "baker-choreofs-wrong-object-demo"
))]
const STAGE_WRONG_OBJECT_REJECTED: u32 = 0x4849_004d;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_TRAP_PATH_OPEN: u32 = 0x4849_0050;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_BEGIN: u32 = 0x4849_0051;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_BORROW_SENT: u32 = 0x4849_0052;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_GRANT_RECV: u32 = 0x4849_0053;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_PATH_DECODED: u32 = 0x4849_0054;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_REQ_SENT: u32 = 0x4849_0055;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_RET_RECV: u32 = 0x4849_0056;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_RELEASE_SENT: u32 = 0x4849_0057;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ENGINE_PATH_OPEN_COMPLETED: u32 = 0x4849_0058;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_PATH_OPEN_BORROW_RECV: u32 = 0x4849_0060;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_PATH_OPEN_GRANT_SENT: u32 = 0x4849_0061;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_PATH_OPEN_REQ_RECV: u32 = 0x4849_0062;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_PATH_OPEN_OBJECT_OPENED: u32 = 0x4849_0063;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_PATH_OPEN_RET_SENT: u32 = 0x4849_0064;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_KERNEL_PATH_OPEN_RELEASE_RECV: u32 = 0x4849_0065;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SLAB_BYTES: usize = 124 * 1024;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const TEST_MEMORY_LEN: u32 = 64 * 1024;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const TEST_MEMORY_EPOCH: u32 = 1;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const TEST_LED_PTR: u32 = 128;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoTransport = SioTransport<Rp2040SioBackend>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type DemoKit = SessionKit<'static, DemoTransport, EngineLabelUniverse, CounterClock, 4>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type KernelEndpoint = Endpoint<'static, 0>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type EngineEndpoint = Endpoint<'static, 1>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type GpioEndpoint = Endpoint<'static, 2>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
type TimerEndpoint = Endpoint<'static, 3>;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(any(
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
type BakerLedger = GuestLedger<4, 1, 1>;
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    not(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    ))
))]
type BakerLedger = GuestLedger<3, 1, 1>;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(not(any(
    feature = "baker-bad-order-demo",
    feature = "baker-chaser-demo",
    feature = "baker-ordinary-std-demo",
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo",
    feature = "baker-invalid-fd-demo",
    feature = "baker-bad-payload-demo"
)))]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-blink.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-bad-order-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-bad-order.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-chaser-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-chaser.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-invalid-fd-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-invalid-fd.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-bad-payload-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-bad-payload.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-ordinary-std-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-ordinary-std-chaser.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-open.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-bad-path-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-bad-path.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-bad-payload-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-bad-payload.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-wrong-object-demo")]
static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-wrong-object.wasm"
));

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_DEMO_RESULT: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_DEMO_FAILURE_STAGE: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
static mut CORE1_STARTED: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
static mut RUNTIME_READY: u32 = 0;

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn mark_stage(value: u32) {
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(HIBANA_DEMO_RESULT), value);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[repr(C)]
struct VectorTable {
    initial_stack_pointer: *const u32,
    reset: unsafe extern "C" fn() -> !,
    exceptions: [timer::IrqHandler; 14],
    timer_irq0: timer::IrqHandler,
    external_irqs: [timer::IrqHandler; 31],
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl Sync for VectorTable {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(link_section = ".vector_table.reset_vector")]
#[used]
static VECTOR_TABLE: VectorTable = VectorTable {
    initial_stack_pointer: core::ptr::addr_of!(__stack_top) as *const u32,
    reset,
    exceptions: [timer::default_irq_handler; 14],
    timer_irq0: timer::timer0_irq_handler,
    external_irqs: [timer::default_irq_handler; 31],
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
struct SharedRuntime {
    clock: CounterClock,
    tap: [TapEvent; 128],
    slab: [u8; SLAB_BYTES],
    session: MaybeUninit<DemoKit>,
    core0_endpoint: MaybeUninit<KernelEndpoint>,
    core1_endpoint: MaybeUninit<EngineEndpoint>,
    core2_endpoint: MaybeUninit<GpioEndpoint>,
    core3_endpoint: MaybeUninit<TimerEndpoint>,
    core1_guest: MaybeUninit<CoreWasip1Instance<'static>>,
    traffic_loop_resolver: BakerTrafficLoopResolver,
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
            core2_endpoint: MaybeUninit::uninit(),
            core3_endpoint: MaybeUninit::uninit(),
            core1_guest: MaybeUninit::uninit(),
            traffic_loop_resolver: BakerTrafficLoopResolver::new(),
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
        0 => {
            init_ram();
            core0_main()
        }
        _ => core1_main(),
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" fn core1_launch_entry() -> ! {
    core1_main()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn init_ram() {
    unsafe {
        let data_src = core::ptr::addr_of!(__data_load_start);
        let data_start = core::ptr::addr_of_mut!(__data_start);
        let data_end = core::ptr::addr_of_mut!(__data_end);
        let data_len = data_end as usize - data_start as usize;
        core::ptr::copy_nonoverlapping(data_src, data_start, data_len);

        let bss_start = core::ptr::addr_of_mut!(__bss_start);
        let bss_end = core::ptr::addr_of_mut!(__bss_end);
        let bss_len = bss_end as usize - bss_start as usize;
        core::ptr::write_bytes(bss_start, 0, bss_len);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core_id() -> u32 {
    unsafe { read_volatile(SIO_CPUID) }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn shared_runtime_ptr() -> *mut SharedRuntime {
    SHARED_RUNTIME.0.get()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe fn shared_kernel_endpoint() -> &'static mut KernelEndpoint {
    unsafe { &mut *(*shared_runtime_ptr()).core0_endpoint.as_mut_ptr() }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe fn shared_engine_endpoint() -> &'static mut EngineEndpoint {
    unsafe { &mut *(*shared_runtime_ptr()).core1_endpoint.as_mut_ptr() }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe fn shared_gpio_endpoint() -> &'static mut GpioEndpoint {
    unsafe { &mut *(*shared_runtime_ptr()).core2_endpoint.as_mut_ptr() }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe fn shared_timer_endpoint() -> &'static mut TimerEndpoint {
    unsafe { &mut *(*shared_runtime_ptr()).core3_endpoint.as_mut_ptr() }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fail_closed(stage: &str) -> ! {
    unsafe {
        let last_stage = read_volatile(core::ptr::addr_of!(HIBANA_DEMO_RESULT));
        write_volatile(
            core::ptr::addr_of_mut!(HIBANA_DEMO_FAILURE_STAGE),
            last_stage,
        );
    }
    mark_stage(RESULT_FAILURE);
    uart::write_bytes(stage.as_bytes());
    uart::write_bytes(b" fail\n");
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fail_at(marker: u32, stage: &str) -> ! {
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(HIBANA_DEMO_FAILURE_STAGE), marker);
    }
    mark_stage(RESULT_FAILURE);
    uart::write_bytes(stage.as_bytes());
    uart::write_bytes(b" fail\n");
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fail_wasm_resume(error: WasmError) -> ! {
    let stage = match error {
        WasmError::Truncated => STAGE_ENGINE_RESUME_ERR_TRUNCATED,
        WasmError::Invalid(_) => STAGE_ENGINE_RESUME_ERR_INVALID,
        WasmError::Unsupported(_) => STAGE_ENGINE_RESUME_ERR_UNSUPPORTED,
        WasmError::UnsupportedOpcode(opcode) => {
            STAGE_ENGINE_RESUME_ERR_OPCODE_BASE | u32::from(opcode)
        }
        WasmError::StackOverflow => STAGE_ENGINE_RESUME_ERR_STACK_OVERFLOW,
        WasmError::StackUnderflow => STAGE_ENGINE_RESUME_ERR_STACK_UNDERFLOW,
        WasmError::PendingHostCall => STAGE_ENGINE_RESUME_ERR_PENDING,
        WasmError::ReplyWithoutPending | WasmError::UnexpectedReply => {
            STAGE_ENGINE_RESUME_ERR_REPLY
        }
        WasmError::Trap => STAGE_ENGINE_RESUME_ERR_TRAP,
        WasmError::FuelExhausted => STAGE_ENGINE_RESUME_ERR_FUEL,
    };
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(HIBANA_DEMO_FAILURE_STAGE), stage);
    }
    mark_stage(RESULT_FAILURE);
    uart::write_bytes(b"[core1] resume core wasip1 traffic guest fail\n");
    park();
}

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    any(
        feature = "baker-bad-order-demo",
        feature = "baker-invalid-fd-demo",
        feature = "baker-bad-payload-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )
))]
fn expected_reject(stage: u32) -> ! {
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(HIBANA_DEMO_FAILURE_STAGE), stage);
    }
    mark_stage(RESULT_EXPECTED_REJECT);
    uart::write_bytes(b"[core1] expected choreography reject\n");
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn handle_gpio_fd_write_error(error: GpioFdWriteError) -> GpioSet {
    #[cfg(feature = "baker-invalid-fd-demo")]
    {
        if error == GpioFdWriteError::BadFd {
            expected_reject(STAGE_INVALID_FD_REJECTED);
        }
    }
    #[cfg(feature = "baker-bad-payload-demo")]
    {
        if error == GpioFdWriteError::BadPayload {
            expected_reject(STAGE_BAD_PAYLOAD_REJECTED);
        }
    }
    #[cfg(feature = "baker-choreofs-bad-payload-demo")]
    {
        if error == GpioFdWriteError::BadPayload {
            expected_reject(STAGE_CHOREOFS_BAD_PAYLOAD_REJECTED);
        }
    }
    #[cfg(feature = "baker-choreofs-wrong-object-demo")]
    {
        if error == GpioFdWriteError::Fd(hibana_pico::kernel::wasi::PicoFdError::WrongResource) {
            expected_reject(STAGE_WRONG_OBJECT_REJECTED);
        }
    }
    let _ = error;
    fail_closed("[core0] resolve led fd_write");
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn attach_or_fail<T>(result: Result<T, AttachError>, stage: &str) -> T {
    match result {
        Ok(value) => value,
        Err(AttachError::Control(CpError::ResourceExhausted)) => fail_closed(stage),
        Err(AttachError::Control(_)) => fail_closed(stage),
        Err(AttachError::Rendezvous(_)) => fail_closed(stage),
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn gpio_ctrl(pin: u8) -> *mut u32 {
    (IO_BANK0_BASE + 0x04 + (pin as usize * 8)) as *mut u32
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn gpio_pad(pin: u8) -> *mut u32 {
    (PADS_BANK0_BASE + 0x04 + (pin as usize * 4)) as *mut u32
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_gpio_bank_init() {
    let reset_mask = RESETS_IO_BANK0 | RESETS_PADS_BANK0;
    unsafe {
        write_volatile(RESETS_RESET_CLR, reset_mask);
        while read_volatile(RESETS_RESET_DONE) & reset_mask != reset_mask {
            core::hint::spin_loop();
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_gpio_init_output(pin: u8, initial_high: bool) {
    let mask = 1u32 << pin;
    unsafe {
        write_volatile(gpio_pad(pin), GPIO_PAD_DEFAULT);
        write_volatile(gpio_ctrl(pin), GPIO_FUNC_SIO);
        if initial_high {
            write_volatile(GPIO_OUT_SET, mask);
        } else {
            write_volatile(GPIO_OUT_CLR, mask);
        }
        write_volatile(GPIO_OE_SET, mask);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_gpio_write(pin: u8, high: bool) {
    let mask = 1u32 << pin;
    unsafe {
        if high {
            write_volatile(GPIO_OUT_SET, mask);
        } else {
            write_volatile(GPIO_OUT_CLR, mask);
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_gpio_apply_baker_led_set(set: GpioSet) {
    apply_baker_link_led_bank_set(rp2040_gpio_write, set);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fifo_drain() {
    while unsafe { read_volatile(SIO_FIFO_ST) } & FIFO_VLD != 0 {
        let _ = unsafe { read_volatile(SIO_FIFO_RD) };
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fifo_clear_errors() {
    unsafe { write_volatile(SIO_FIFO_ST_WRITE, FIFO_WOF | FIFO_ROE) };
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fifo_push_blocking(word: u32) {
    while unsafe { read_volatile(SIO_FIFO_ST) } & FIFO_RDY == 0 {
        core::hint::spin_loop();
    }
    unsafe { write_volatile(SIO_FIFO_WR, word) };
    sev();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn fifo_pop_blocking() -> u32 {
    while unsafe { read_volatile(SIO_FIFO_ST) } & FIFO_VLD == 0 {
        core::hint::spin_loop();
    }
    unsafe { read_volatile(SIO_FIFO_RD) }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn sev() {
    unsafe { asm!("sev", options(nomem, nostack, preserves_flags)) };
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn ensure_core1_launched() {
    for _ in 0..100_000 {
        if unsafe { read_volatile(core::ptr::addr_of!(CORE1_STARTED)) } != 0 {
            return;
        }
        core::hint::spin_loop();
    }

    let sequence = [
        0,
        0,
        1,
        core::ptr::addr_of!(VECTOR_TABLE) as u32,
        core::ptr::addr_of!(__core1_stack_top) as u32,
        core1_launch_entry as *const () as usize as u32,
    ];
    for &word in sequence.iter() {
        loop {
            fifo_drain();
            fifo_clear_errors();
            sev();
            fifo_push_blocking(word);
            if fifo_pop_blocking() == word {
                break;
            }
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn init_runtime_once() {
    mark_stage(STAGE_RUNTIME_BEGIN);
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
        mark_stage(STAGE_RENDEZVOUS_READY);
        let sid = SessionId::new(10);
        #[cfg(any(
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        ))]
        let (core0_program, core1_program, core2_program, core3_program) =
            baker_led_choreofs_blink_roles();
        #[cfg(not(any(
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )))]
        let (core0_program, core1_program, core2_program, core3_program) = baker_led_blink_roles();
        mark_stage(STAGE_PROGRAM_READY);
        if kit
            .set_resolver::<POLICY_BAKER_TRAFFIC_LOOP, 1>(
                rv,
                &core1_program,
                ResolverRef::loop_state(
                    &(*runtime).traffic_loop_resolver,
                    BakerTrafficLoopResolver::resolve_policy,
                ),
            )
            .is_err()
        {
            fail_closed("[core0] register traffic loop resolver");
        }
        (*runtime).core0_endpoint.as_mut_ptr().write(attach_or_fail(
            kit.enter(rv, sid, &core0_program, NoBinding),
            "[core0] attach endpoint",
        ));
        mark_stage(STAGE_KERNEL_ATTACHED);
        (*runtime).core1_endpoint.as_mut_ptr().write(attach_or_fail(
            kit.enter(rv, sid, &core1_program, NoBinding),
            "[core1] attach endpoint",
        ));
        mark_stage(STAGE_ENGINE_ATTACHED);
        (*runtime).core2_endpoint.as_mut_ptr().write(attach_or_fail(
            kit.enter(rv, sid, &core2_program, NoBinding),
            "[core0] attach gpio endpoint",
        ));
        mark_stage(STAGE_GPIO_ATTACHED);
        (*runtime).core3_endpoint.as_mut_ptr().write(attach_or_fail(
            kit.enter(rv, sid, &core3_program, NoBinding),
            "[core0] attach timer endpoint",
        ));
        write_volatile(core::ptr::addr_of_mut!(RUNTIME_READY), 1);
    }
    signal();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn gpio_device_offer_set_once(endpoint: &mut GpioEndpoint) {
    let branch = match endpoint.offer().await {
        Ok(branch) => branch,
        Err(_) => fail_closed("[gpio] offer set"),
    };
    if branch.label() != LABEL_GPIO_SET {
        fail_closed("[gpio] set label mismatch");
    }
    let set = match branch.decode::<Msg<LABEL_GPIO_SET, GpioSet>>().await {
        Ok(set) => set,
        Err(_) => fail_closed("[gpio] decode set"),
    };
    rp2040_gpio_apply_baker_led_set(set);
    match endpoint
        .flow::<Msg<LABEL_GPIO_SET_DONE, GpioSet>>()
        .expect("gpio flow<set done>")
        .send(&set)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[gpio] send set done"),
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn timer_device_offer_sleep_once(
    endpoint: &mut TimerEndpoint,
    resolver: &mut PicoInterruptResolver<2, 4, 1>,
    delay_ticks: u32,
) {
    let branch = match endpoint.offer().await {
        Ok(branch) => branch,
        Err(_) => fail_closed("[timer] offer sleep"),
    };
    if branch.label() != LABEL_TIMER_SLEEP_UNTIL {
        fail_closed("[timer] sleep label mismatch");
    }
    let sleep = match branch
        .decode::<Msg<LABEL_TIMER_SLEEP_UNTIL, TimerSleepUntil>>()
        .await
    {
        Ok(sleep) => sleep,
        Err(_) => fail_closed("[timer] decode sleep"),
    };
    mark_stage(STAGE_TIMER_SLEEP_RECV);

    resolver
        .request_timer_sleep(sleep)
        .unwrap_or_else(|_| fail_closed("[timer] resolver register sleep"));
    resolver
        .push_irq(InterruptEvent::TimerTick {
            tick: sleep.tick().saturating_sub(1),
        })
        .unwrap_or_else(|_| fail_closed("[timer] resolver record early tick"));
    if resolver
        .resolve_next()
        .unwrap_or_else(|_| fail_closed("[timer] resolver early tick resolve"))
        .is_some()
    {
        fail_closed("[timer] completed before due tick");
    }

    timer::arm_alarm0_after_ticks(delay_ticks);
    mark_stage(STAGE_TIMER_ALARM_ARMED);
    wait_until(timer::alarm0_ready);
    mark_stage(STAGE_TIMER_RAW_READY);
    let Some(_ready) = timer::take_alarm0_ready() else {
        fail_closed("[timer] raw readiness vanished");
    };
    resolver
        .push_irq(InterruptEvent::TimerTick { tick: sleep.tick() })
        .unwrap_or_else(|_| fail_closed("[timer] resolver record due tick"));
    let Some(ResolvedInterrupt::TimerSleepDone(done)) = resolver
        .resolve_next()
        .unwrap_or_else(|_| fail_closed("[timer] resolver due tick resolve"))
    else {
        fail_closed("[timer] expected timer sleep done");
    };
    if done.tick() != sleep.tick() {
        fail_closed("[timer] sleep done mismatch");
    }

    match endpoint
        .flow::<Msg<LABEL_TIMER_SLEEP_DONE, TimerSleepDone>>()
        .expect("timer flow<sleep done>")
        .send(&done)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[timer] send sleep done"),
    }
    mark_stage(STAGE_TIMER_DONE_SENT);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn kernel_fd_write(
    endpoint: &mut KernelEndpoint,
    gpio_endpoint: &mut GpioEndpoint,
    ledger: &mut BakerLedger,
    borrow: MemBorrow,
) {
    mark_stage(STAGE_KERNEL_FD_WRITE_BEGIN);
    mark_stage(STAGE_KERNEL_FD_WRITE_BORROW_RECV);
    if borrow.ptr() != TEST_LED_PTR
        || borrow.len() == 0
        || borrow.len() as usize > WASIP1_STREAM_CHUNK_CAPACITY
        || borrow.epoch() != TEST_MEMORY_EPOCH
    {
        fail_closed("[core0] led mem borrow mismatch");
    }
    let grant = ledger
        .grant_read_lease(borrow)
        .unwrap_or_else(|_| fail_closed("[core0] grant led read lease"));
    match endpoint
        .flow::<MemReadGrantControl>()
        .expect("kernel flow<led grant>")
        .send(())
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send led mem grant"),
    }
    mark_stage(STAGE_KERNEL_FD_WRITE_GRANT_SENT);

    let request = match endpoint.recv::<Msg<LABEL_WASI_FD_WRITE, EngineReq>>().await {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv led fd_write"),
    };
    mark_stage(STAGE_KERNEL_FD_WRITE_REQ_RECV);
    let EngineReq::FdWrite(write) = request else {
        fail_closed("[core0] expected led fd_write");
    };
    if ledger.validate_fd_write_lease(&write, grant).is_err() {
        fail_closed("[core0] led fd_write lease mismatch");
    }
    let set = check_gpio_object_fd_write(ledger.fd_view(), &write, baker_link_led_fd_write_route())
        .unwrap_or_else(handle_gpio_fd_write_error);
    match endpoint
        .flow::<Msg<LABEL_GPIO_SET, GpioSet>>()
        .expect("kernel flow<gpio set>")
        .send(&set)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send gpio set"),
    }
    gpio_device_offer_set_once(gpio_endpoint).await;
    let done = match endpoint.recv::<Msg<LABEL_GPIO_SET_DONE, GpioSet>>().await {
        Ok(done) => done,
        Err(_) => fail_closed("[core0] recv gpio set done"),
    };
    if done != set {
        fail_closed("[core0] gpio set done mismatch");
    }
    mark_stage(STAGE_KERNEL_FD_WRITE_GPIO_DONE);

    let reply = EngineRet::FdWriteDone(FdWriteDone::new(write.fd(), write.len() as u8));
    match endpoint
        .flow::<Msg<LABEL_WASI_FD_WRITE_RET, EngineRet>>()
        .expect("kernel flow<led fd_write ret>")
        .send(&reply)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send led fd_write ret"),
    }

    let release = match endpoint.recv::<Msg<LABEL_MEM_RELEASE, MemRelease>>().await {
        Ok(release) => release,
        Err(_) => fail_closed("[core0] recv led mem release"),
    };
    if release.lease_id() != grant.lease_id() {
        fail_closed("[core0] led release mismatch");
    }
    ledger
        .release_lease(release)
        .unwrap_or_else(|_| fail_closed("[core0] release led lease"));
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn kernel_path_open(
    endpoint: &mut KernelEndpoint,
    ledger: &mut BakerLedger,
    store: &BakerLinkLedResourceStore,
    borrow: MemBorrow,
) {
    mark_stage(STAGE_KERNEL_PATH_OPEN_BORROW_RECV);
    if borrow.len() == 0 || borrow.epoch() != TEST_MEMORY_EPOCH {
        fail_closed("[core0] path_open borrow mismatch");
    }
    let grant = ledger
        .grant_read_lease(borrow)
        .unwrap_or_else(|_| fail_closed("[core0] grant path_open read lease"));
    match endpoint
        .flow::<MemReadGrantControl>()
        .expect("kernel flow<path grant>")
        .send(())
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send path_open mem grant"),
    }
    mark_stage(STAGE_KERNEL_PATH_OPEN_GRANT_SENT);

    let request = match endpoint
        .recv::<Msg<LABEL_WASI_PATH_OPEN, EngineReq>>()
        .await
    {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv path_open"),
    };
    let EngineReq::PathOpen(open) = request else {
        fail_closed("[core0] expected path_open");
    };
    mark_stage(STAGE_KERNEL_PATH_OPEN_REQ_RECV);
    if open.preopen_fd() != BAKER_LINK_CHOREOFS_PREOPEN_FD
        || open.lease_id() != grant.lease_id()
        || open.len() > borrow.len() as usize
    {
        fail_closed("[core0] path_open lease mismatch");
    }

    let opened = match open_baker_link_choreofs_path(store, ledger, open.path(), open.rights_base())
    {
        Ok(opened) => opened,
        Err(error) => {
            handle_baker_choreofs_open_error(error);
        }
    };
    mark_stage(STAGE_KERNEL_PATH_OPEN_OBJECT_OPENED);
    let reply = EngineRet::PathOpened(PathOpened::new(opened.fd(), 0));
    match endpoint
        .flow::<Msg<LABEL_WASI_PATH_OPEN_RET, EngineRet>>()
        .expect("kernel flow<path_open ret>")
        .send(&reply)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send path_open ret"),
    }
    mark_stage(STAGE_KERNEL_PATH_OPEN_RET_SENT);

    let release = match endpoint.recv::<Msg<LABEL_MEM_RELEASE, MemRelease>>().await {
        Ok(release) => release,
        Err(_) => fail_closed("[core0] recv path_open mem release"),
    };
    mark_stage(STAGE_KERNEL_PATH_OPEN_RELEASE_RECV);
    if release.lease_id() != grant.lease_id() {
        fail_closed("[core0] path_open release mismatch");
    }
    ledger
        .release_lease(release)
        .unwrap_or_else(|_| fail_closed("[core0] release path_open lease"));
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn handle_baker_choreofs_open_error(error: ChoreoFsError) -> ! {
    #[cfg(feature = "baker-choreofs-bad-path-demo")]
    {
        if matches!(error, ChoreoFsError::NotFound | ChoreoFsError::AbsolutePath) {
            expected_reject(STAGE_BAD_PATH_REJECTED);
        }
    }
    let _ = error;
    fail_closed("[core0] resolve path_open");
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn kernel_poll_oneoff(
    endpoint: &mut KernelEndpoint,
    timer_endpoint: &mut TimerEndpoint,
    ledger: &mut BakerLedger,
    resolver: &mut PicoInterruptResolver<2, 4, 1>,
    last_tick: &mut u64,
) {
    let request = match endpoint
        .recv::<Msg<LABEL_WASI_POLL_ONEOFF, EngineReq>>()
        .await
    {
        Ok(request) => request,
        Err(_) => fail_closed("[core0] recv led poll_oneoff"),
    };
    mark_stage(STAGE_KERNEL_POLL_RECV);
    let EngineReq::PollOneoff(poll) = request else {
        fail_closed("[core0] expected led poll_oneoff");
    };
    if poll.timeout_tick() < *last_tick {
        fail_closed("[core0] led poll_oneoff tick moved backwards");
    }
    let pending_poll = ledger
        .begin_poll_oneoff(poll)
        .unwrap_or_else(|_| fail_closed("[core0] begin pending poll_oneoff"));
    let delta = poll.timeout_tick() - *last_tick;
    if delta > u32::MAX as u64 {
        fail_closed("[core0] led poll_oneoff delta too large");
    }
    let delay_ticks = delta as u32;
    *last_tick = poll.timeout_tick();

    let sleep = TimerSleepUntil::new(poll.timeout_tick());
    match endpoint
        .flow::<Msg<LABEL_TIMER_SLEEP_UNTIL, TimerSleepUntil>>()
        .expect("kernel flow<timer sleep>")
        .send(&sleep)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send timer sleep"),
    }
    mark_stage(STAGE_KERNEL_TIMER_SLEEP_SENT);
    timer_device_offer_sleep_once(timer_endpoint, resolver, delay_ticks).await;
    let done = match endpoint
        .recv::<Msg<LABEL_TIMER_SLEEP_DONE, TimerSleepDone>>()
        .await
    {
        Ok(done) => done,
        Err(_) => fail_closed("[core0] recv timer sleep done"),
    };
    if done.tick() != poll.timeout_tick() {
        fail_closed("[core0] resolver timer tick mismatch");
    }
    ledger
        .complete_poll_oneoff(pending_poll, done)
        .unwrap_or_else(|_| fail_closed("[core0] complete pending poll_oneoff"));

    let reply = EngineRet::PollReady(PollReady::new(1));
    match endpoint
        .flow::<Msg<LABEL_WASI_POLL_ONEOFF_RET, EngineRet>>()
        .expect("kernel flow<led poll_oneoff ret>")
        .send(&reply)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core0] send led poll_oneoff ret"),
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn kernel_session(
    endpoint: &mut KernelEndpoint,
    gpio_endpoint: &mut GpioEndpoint,
    timer_endpoint: &mut TimerEndpoint,
) {
    #[cfg(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    ))]
    let store = baker_link_led_resource_store()
        .unwrap_or_else(|_| fail_closed("[core0] create baker choreofs store"));
    #[cfg(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    ))]
    let mut ledger =
        baker_link_choreofs_ledger::<4, 1, 1>(&store, TEST_MEMORY_LEN, TEST_MEMORY_EPOCH)
            .unwrap_or_else(|_| fail_closed("[core0] create baker choreofs ledger"));
    #[cfg(not(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )))]
    let mut ledger = hibana_pico::machine::rp2040::baker_link::baker_link_pico_min_ledger::<1, 1>(
        TEST_MEMORY_LEN,
        TEST_MEMORY_EPOCH,
    )
    .unwrap_or_else(|_| fail_closed("[core0] create baker pico-min ledger"));
    let mut resolver: PicoInterruptResolver<2, 4, 1> = PicoInterruptResolver::new();

    let activation_id = 0u16;
    kernel_start_app_activation(endpoint, activation_id, 0).await;
    #[cfg(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    ))]
    for _ in 0..3 {
        let borrow = endpoint
            .recv::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
            .await
            .unwrap_or_else(|_| fail_closed("[core0] recv path_open mem borrow"));
        kernel_path_open(endpoint, &mut ledger, &store, borrow).await;
    }
    let mut step = 0usize;
    let mut tick = 0u64;
    loop {
        let Some(borrow) = kernel_next_app_step_or_exit(endpoint).await else {
            break;
        };
        kernel_fd_write(endpoint, gpio_endpoint, &mut ledger, borrow).await;
        if step == 0 {
            mark_stage(STAGE_FIRST_LED_WRITE_DONE);
        }
        kernel_poll_oneoff(
            endpoint,
            timer_endpoint,
            &mut ledger,
            &mut resolver,
            &mut tick,
        )
        .await;
        if step == 0 {
            mark_stage(STAGE_POLL_ON_DONE);
        }
        mark_stage(STAGE_FINAL_LED_WRITE_DONE);
        step += 1;
    }
    mark_stage(RESULT_SUCCESS);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn kernel_start_app_activation(endpoint: &mut KernelEndpoint, activation_id: u16, tick: u64) {
    mark_stage(STAGE_KERNEL_RUN_SEND_BEGIN);
    let run = BudgetRun::new(
        activation_id,
        1,
        BAKER_LINK_TRAFFIC_LIGHT_PATTERN_STEPS as u32,
        tick,
    );
    let flow = endpoint
        .flow::<BudgetRunMsg>()
        .unwrap_or_else(|_| fail_at(STAGE_KERNEL_RUN_FLOW_ERR, "[core0] flow traffic run"));
    match flow.send(&run).await {
        Ok(_) => {}
        Err(_) => fail_at(STAGE_KERNEL_RUN_SEND_ERR, "[core0] send traffic run"),
    }
    mark_stage(STAGE_KERNEL_RUN_SEND_DONE);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn kernel_next_app_step_or_exit(endpoint: &mut KernelEndpoint) -> Option<MemBorrow> {
    let branch = match endpoint.offer().await {
        Ok(branch) => branch,
        Err(_) => fail_closed("[core0] offer next app step"),
    };
    match branch.label() {
        LABEL_MEM_BORROW_READ => {
            let borrow = match branch
                .decode::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
                .await
            {
                Ok(borrow) => borrow,
                Err(_) => fail_closed("[core0] decode next mem borrow"),
            };
            if borrow.ptr() != TEST_LED_PTR
                || borrow.len() == 0
                || borrow.len() as usize > WASIP1_STREAM_CHUNK_CAPACITY
                || borrow.epoch() != TEST_MEMORY_EPOCH
            {
                fail_closed("[core0] next mem borrow mismatch");
            }
            Some(borrow)
        }
        LABEL_WASI_PROC_EXIT => {
            let request = match branch
                .decode::<Msg<LABEL_WASI_PROC_EXIT, EngineReq>>()
                .await
            {
                Ok(request) => request,
                Err(_) => fail_closed("[core0] decode proc_exit"),
            };
            let EngineReq::ProcExit(status) = request else {
                fail_closed("[core0] expected proc_exit");
            };
            if status.code() != 0 {
                fail_closed("[core0] nonzero proc_exit");
            }
            mark_stage(STAGE_KERNEL_PROC_EXIT_RECV);
            None
        }
        _ => fail_closed("[core0] unexpected app step label"),
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_fd_write(endpoint: &mut EngineEndpoint, fd: u8, payload: &[u8]) {
    mark_stage(STAGE_ENGINE_FD_WRITE_BEGIN);
    let borrow = MemBorrow::new(TEST_LED_PTR, payload.len() as u8, TEST_MEMORY_EPOCH);
    match endpoint
        .flow::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
        .expect("engine flow<led borrow>")
        .send(&borrow)
        .await
    {
        Ok(_) => {}
        Err(_) => {
            mark_stage(STAGE_ENGINE_BORROW_SEND_ERR);
            fail_closed("[core1] send led mem borrow");
        }
    }
    mark_stage(STAGE_ENGINE_FD_WRITE_BORROW_SENT);

    let grant = match endpoint.recv::<MemReadGrantControl>().await {
        Ok(grant) => grant,
        Err(_) => fail_closed("[core1] recv led mem grant"),
    };
    let (rights, lease_id) = grant
        .decode_handle()
        .unwrap_or_else(|_| fail_closed("[core1] decode led mem grant"));
    if rights != MemRights::Read.tag() || lease_id > u8::MAX as u64 {
        fail_closed("[core1] led mem grant mismatch");
    }

    let write = FdWrite::new_with_lease(fd, lease_id as u8, payload)
        .unwrap_or_else(|_| fail_closed("[core1] make led fd_write"));
    let request = EngineReq::FdWrite(write);
    match endpoint
        .flow::<Msg<LABEL_WASI_FD_WRITE, EngineReq>>()
        .expect("engine flow<led fd_write>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send led fd_write"),
    }

    let reply = match endpoint
        .recv::<Msg<LABEL_WASI_FD_WRITE_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv led fd_write ret"),
    };
    let EngineRet::FdWriteDone(done) = reply else {
        fail_closed("[core1] expected led fd_write ret");
    };
    if done.fd() != fd || done.written() != payload.len() as u8 {
        fail_closed("[core1] led fd_write ret mismatch");
    }

    let release = MemRelease::new(lease_id as u8);
    match endpoint
        .flow::<Msg<LABEL_MEM_RELEASE, MemRelease>>()
        .expect("engine flow<led release>")
        .send(&release)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send led mem release"),
    }
}

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    not(feature = "baker-bad-order-demo")
))]
async fn engine_poll_oneoff(endpoint: &mut EngineEndpoint, tick: u64) {
    let request = EngineReq::PollOneoff(PollOneoff::new(tick));
    match endpoint
        .flow::<Msg<LABEL_WASI_POLL_ONEOFF, EngineReq>>()
        .expect("engine flow<led poll_oneoff>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send led poll_oneoff"),
    }

    let reply = match endpoint
        .recv::<Msg<LABEL_WASI_POLL_ONEOFF_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv led poll_oneoff ret"),
    };
    let EngineRet::PollReady(ready) = reply else {
        fail_closed("[core1] expected led poll_oneoff ret");
    };
    if ready.ready() != 1 {
        fail_closed("[core1] led poll_oneoff ready mismatch");
    }
}

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "baker-bad-order-demo"
))]
fn engine_expect_poll_oneoff_rejected(endpoint: &mut EngineEndpoint) -> ! {
    if endpoint
        .flow::<Msg<LABEL_WASI_POLL_ONEOFF, EngineReq>>()
        .is_ok()
    {
        fail_closed("[core1] bad-order poll unexpectedly accepted");
    }
    expected_reject(STAGE_BAD_ORDER_POLL_REJECTED);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_path_open(
    endpoint: &mut EngineEndpoint,
    guest: &mut CoreWasip1Instance<'static>,
    call: CoreWasip1PathCall,
) {
    mark_stage(STAGE_ENGINE_PATH_OPEN_BEGIN);
    if call.kind() != CoreWasip1PathKind::PathOpen {
        fail_closed("[core1] expected path_open");
    }
    let ptr = call
        .arg_i32(2)
        .unwrap_or_else(|_| fail_closed("[core1] path_open ptr"));
    let len = call
        .arg_i32(3)
        .unwrap_or_else(|_| fail_closed("[core1] path_open len"));
    if len > u8::MAX as u32 {
        fail_closed("[core1] path_open path too long");
    }
    let preopen_fd = call
        .fd()
        .unwrap_or_else(|_| fail_closed("[core1] path_open preopen fd"));
    let rights_base = call
        .arg_i64(5)
        .unwrap_or_else(|_| fail_closed("[core1] path_open rights"));

    let borrow = MemBorrow::new(ptr, len as u8, TEST_MEMORY_EPOCH);
    match endpoint
        .flow::<Msg<LABEL_MEM_BORROW_READ, MemBorrow>>()
        .expect("engine flow<path borrow>")
        .send(&borrow)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send path_open mem borrow"),
    }
    mark_stage(STAGE_ENGINE_PATH_OPEN_BORROW_SENT);
    let grant = match endpoint.recv::<MemReadGrantControl>().await {
        Ok(grant) => grant,
        Err(_) => fail_closed("[core1] recv path_open mem grant"),
    };
    mark_stage(STAGE_ENGINE_PATH_OPEN_GRANT_RECV);
    let (rights, lease_id) = grant
        .decode_handle()
        .unwrap_or_else(|_| fail_closed("[core1] decode path_open mem grant"));
    if rights != MemRights::Read.tag() || lease_id > u8::MAX as u64 {
        fail_closed("[core1] path_open grant mismatch");
    }

    let path = guest
        .path_bytes(call)
        .unwrap_or_else(|_| fail_closed("[core1] decode path_open path"));
    mark_stage(STAGE_ENGINE_PATH_OPEN_PATH_DECODED);
    let request = EngineReq::PathOpen(
        PathOpen::new(preopen_fd, lease_id as u8, rights_base, path.as_bytes())
            .unwrap_or_else(|_| fail_closed("[core1] make path_open request")),
    );
    match endpoint
        .flow::<Msg<LABEL_WASI_PATH_OPEN, EngineReq>>()
        .expect("engine flow<path_open>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send path_open"),
    }
    mark_stage(STAGE_ENGINE_PATH_OPEN_REQ_SENT);
    let reply = match endpoint
        .recv::<Msg<LABEL_WASI_PATH_OPEN_RET, EngineRet>>()
        .await
    {
        Ok(reply) => reply,
        Err(_) => fail_closed("[core1] recv path_open ret"),
    };
    let EngineRet::PathOpened(opened) = reply else {
        fail_closed("[core1] expected path_open ret");
    };
    mark_stage(STAGE_ENGINE_PATH_OPEN_RET_RECV);

    let release = MemRelease::new(lease_id as u8);
    match endpoint
        .flow::<Msg<LABEL_MEM_RELEASE, MemRelease>>()
        .expect("engine flow<path release>")
        .send(&release)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send path_open mem release"),
    }
    mark_stage(STAGE_ENGINE_PATH_OPEN_RELEASE_SENT);

    guest
        .complete_path_open(call, opened.fd() as u32, opened.errno() as u32)
        .unwrap_or_else(|_| fail_closed("[core1] complete path_open"));
    mark_stage(STAGE_ENGINE_PATH_OPEN_COMPLETED);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_session(endpoint: &mut EngineEndpoint, guest: &mut CoreWasip1Instance<'static>) {
    #[cfg(not(feature = "baker-bad-order-demo"))]
    let mut tick = 0u64;
    engine_recv_traffic_run(endpoint, 0).await;
    loop {
        let trap = match guest.resume() {
            Ok(trap) => trap,
            Err(error) => fail_wasm_resume(error),
        };
        match trap {
            CoreWasip1Trap::FdWrite(call) => {
                mark_stage(STAGE_ENGINE_TRAP_FD_WRITE);
                engine_continue_traffic_loop(endpoint).await;
                let payload = guest
                    .fd_write_payload(call)
                    .unwrap_or_else(|_| fail_closed("[core1] decode wasip1 fd_write iovec"));
                engine_fd_write(endpoint, call.fd(), payload.as_bytes()).await;
                guest
                    .complete_fd_write(call, 0)
                    .unwrap_or_else(|_| fail_closed("[core1] complete wasip1 fd_write"));
            }
            CoreWasip1Trap::PollOneoff(call) => {
                mark_stage(STAGE_ENGINE_TRAP_POLL_ONEOFF);
                let delay_ticks = guest
                    .poll_oneoff_delay_ticks(call)
                    .unwrap_or_else(|_| fail_closed("[core1] decode wasip1 poll_oneoff"));
                #[cfg(feature = "baker-bad-order-demo")]
                {
                    if delay_ticks != 50 {
                        fail_closed("[core1] bad-order poll delay mismatch");
                    }
                    engine_expect_poll_oneoff_rejected(endpoint);
                }
                #[cfg(not(feature = "baker-bad-order-demo"))]
                {
                    tick = tick.saturating_add(delay_ticks);
                    engine_poll_oneoff(endpoint, tick).await;
                    guest
                        .complete_poll_oneoff(call, 1, 0)
                        .unwrap_or_else(|_| fail_closed("[core1] complete wasip1 poll_oneoff"));
                }
            }
            CoreWasip1Trap::FdRead(_)
            | CoreWasip1Trap::FdFdstatGet(_)
            | CoreWasip1Trap::FdClose(_)
            | CoreWasip1Trap::ClockResGet(_)
            | CoreWasip1Trap::ClockTimeGet(_)
            | CoreWasip1Trap::RandomGet(_)
            | CoreWasip1Trap::SchedYield
            | CoreWasip1Trap::PathFull(_)
            | CoreWasip1Trap::Socket(_)
            | CoreWasip1Trap::ProcRaise(_) => {
                mark_stage(STAGE_ENGINE_TRAP_UNSUPPORTED);
                fail_closed("[core1] unsupported wasip1 syscall in pico-min profile");
            }
            CoreWasip1Trap::PathMinimal(call) => {
                mark_stage(STAGE_ENGINE_TRAP_PATH_OPEN);
                engine_path_open(endpoint, guest, call).await;
            }
            CoreWasip1Trap::ArgsSizesGet(call) => {
                mark_stage(STAGE_ENGINE_TRAP_ARGS_SIZES);
                let _ = call;
                fail_closed("[core1] unexpected args_sizes_get in Baker profile");
            }
            CoreWasip1Trap::ArgsGet(call) => {
                mark_stage(STAGE_ENGINE_TRAP_ARGS_GET);
                let _ = call;
                fail_closed("[core1] unexpected args_get in Baker profile");
            }
            CoreWasip1Trap::EnvironSizesGet(call) => {
                mark_stage(STAGE_ENGINE_TRAP_ENVIRON_SIZES);
                let _ = call;
                fail_closed("[core1] unexpected environ_sizes_get in Baker choreography");
            }
            CoreWasip1Trap::EnvironGet(call) => {
                mark_stage(STAGE_ENGINE_TRAP_ENVIRON_GET);
                let _ = call;
                fail_closed("[core1] unexpected environ_get in Baker choreography");
            }
            CoreWasip1Trap::ProcExit(status) => {
                if status > u8::MAX as u32 {
                    fail_closed("[core1] proc_exit status too large");
                }
                engine_break_traffic_loop(endpoint).await;
                engine_proc_exit(endpoint, status as u8).await;
                break;
            }
            CoreWasip1Trap::MemoryGrow(_) => {
                mark_stage(STAGE_ENGINE_TRAP_MEMORY_GROW);
                fail_closed("[core1] unexpected memory.grow in pico-min profile");
            }
            CoreWasip1Trap::Done => {
                engine_break_traffic_loop(endpoint).await;
                engine_proc_exit(endpoint, 0).await;
                break;
            }
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_continue_traffic_loop(endpoint: &mut EngineEndpoint) {
    match endpoint
        .flow::<BakerTrafficLoopContinueControl>()
        .expect("engine flow<traffic loop continue>")
        .send(())
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send traffic loop continue"),
    }
    mark_stage(STAGE_ENGINE_LOOP_CONTINUE_SENT);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_break_traffic_loop(endpoint: &mut EngineEndpoint) {
    match endpoint
        .flow::<BakerTrafficLoopBreakControl>()
        .expect("engine flow<traffic loop break>")
        .send(())
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send traffic loop break"),
    }
    mark_stage(STAGE_ENGINE_LOOP_BREAK_SENT);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_proc_exit(endpoint: &mut EngineEndpoint, code: u8) {
    let request = EngineReq::ProcExit(ProcExitStatus::new(code));
    match endpoint
        .flow::<Msg<LABEL_WASI_PROC_EXIT, EngineReq>>()
        .expect("engine flow<proc_exit>")
        .send(&request)
        .await
    {
        Ok(_) => {}
        Err(_) => fail_closed("[core1] send proc_exit"),
    }
    mark_stage(STAGE_ENGINE_PROC_EXIT_SENT);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
async fn engine_recv_traffic_run(endpoint: &mut EngineEndpoint, expected_cycle: u16) {
    mark_stage(STAGE_ENGINE_RUN_RECV_BEGIN);
    let run = match endpoint.recv::<BudgetRunMsg>().await {
        Ok(run) => run,
        Err(_) => fail_at(STAGE_ENGINE_RUN_RECV_ERR, "[core1] recv traffic run"),
    };
    if run.run_id() != expected_cycle
        || run.generation() != 1
        || run.fuel() != BAKER_LINK_TRAFFIC_LIGHT_PATTERN_STEPS as u32
    {
        fail_at(STAGE_ENGINE_RUN_MISMATCH, "[core1] traffic run mismatch");
    }
    mark_stage(STAGE_ENGINE_RUN_RECV_DONE);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core0_main() -> ! {
    mark_stage(STAGE_CORE0_START);
    let _ = clock::init_125mhz();
    uart::init();
    ensure_core1_launched();
    fifo_drain();
    mark_stage(STAGE_CORE1_LAUNCHED);
    rp2040_gpio_bank_init();
    for pin in BAKER_LINK_LED_PINS {
        rp2040_gpio_init_output(pin, !BAKER_LINK_LED_ACTIVE_HIGH);
    }
    mark_stage(STAGE_GPIO_READY);
    init_runtime_once();
    mark_stage(STAGE_RUNTIME_READY);
    let endpoint = unsafe { shared_kernel_endpoint() };
    let gpio_endpoint = unsafe { shared_gpio_endpoint() };
    let timer_endpoint = unsafe { shared_timer_endpoint() };
    run_current_task(kernel_session(endpoint, gpio_endpoint, timer_endpoint));
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn core1_main() -> ! {
    fifo_drain();
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(CORE1_STARTED), 1);
    }
    signal();
    wait_until(uart::ready);
    wait_until(|| unsafe { read_volatile(core::ptr::addr_of!(RUNTIME_READY)) } != 0);
    mark_stage(STAGE_ENGINE_RUNTIME_READY_SEEN);
    let endpoint = unsafe { shared_engine_endpoint() };
    mark_stage(STAGE_ENGINE_ENDPOINT_READY);
    mark_stage(STAGE_ENGINE_BEGIN);
    let guest_slot = unsafe { &mut (*shared_runtime_ptr()).core1_guest };
    let guest = CoreWasip1Instance::write_new_in_place(
        WASIP1_LED_GUEST,
        baker_wasip1_handler_set(),
        guest_slot,
    )
    .unwrap_or_else(|_| fail_closed("[core1] instantiate core wasip1 traffic guest"));
    mark_stage(STAGE_ENGINE_PARSE_DONE);
    run_current_task(engine_session(endpoint, guest));
    park();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
const fn baker_wasip1_handler_set() -> Wasip1HandlerSet {
    if cfg!(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )) {
        Wasip1HandlerSet::PICO_STD_CHOREOFS
    } else if cfg!(feature = "baker-ordinary-std-demo") {
        Wasip1HandlerSet::PICO_STD_START
    } else {
        Wasip1HandlerSet::PICO_MIN
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    fail_closed("[panic]")
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}
