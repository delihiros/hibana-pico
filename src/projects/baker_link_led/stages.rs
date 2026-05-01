pub const RESULT_SUCCESS: u32 = 0x4849_4f4b;
#[cfg(any(
    feature = "baker-bad-order-demo",
    feature = "baker-invalid-fd-demo",
    feature = "baker-bad-payload-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
pub const RESULT_EXPECTED_REJECT: u32 = 0x4849_524a;
pub const RESULT_FAILURE: u32 = 0x4849_4641;

pub const STAGE_CORE0_START: u32 = 0x4849_0001;
pub const STAGE_CORE1_LAUNCHED: u32 = 0x4849_0002;
pub const STAGE_GPIO_READY: u32 = 0x4849_0003;
pub const STAGE_RUNTIME_BEGIN: u32 = 0x4849_0004;
pub const STAGE_RENDEZVOUS_READY: u32 = 0x4849_0005;
pub const STAGE_PROGRAM_READY: u32 = 0x4849_0006;
pub const STAGE_KERNEL_ATTACHED: u32 = 0x4849_0007;
pub const STAGE_ENGINE_ATTACHED: u32 = 0x4849_0008;
pub const STAGE_GPIO_ATTACHED: u32 = 0x4849_0009;
pub const STAGE_RUNTIME_READY: u32 = 0x4849_000a;
pub const STAGE_FIRST_LED_WRITE_DONE: u32 = 0x4849_000b;
pub const STAGE_POLL_ON_DONE: u32 = 0x4849_000c;
pub const STAGE_FINAL_LED_WRITE_DONE: u32 = 0x4849_000d;
pub const STAGE_KERNEL_POLL_RECV: u32 = 0x4849_0010;
pub const STAGE_KERNEL_TIMER_SLEEP_SENT: u32 = 0x4849_0011;
pub const STAGE_TIMER_SLEEP_RECV: u32 = 0x4849_0012;
pub const STAGE_TIMER_ALARM_ARMED: u32 = 0x4849_0013;
pub const STAGE_TIMER_RAW_READY: u32 = 0x4849_0014;
pub const STAGE_TIMER_DONE_SENT: u32 = 0x4849_0015;
pub const STAGE_KERNEL_FD_WRITE_BEGIN: u32 = 0x4849_0020;
pub const STAGE_KERNEL_FD_WRITE_BORROW_RECV: u32 = 0x4849_0021;
pub const STAGE_KERNEL_FD_WRITE_GRANT_SENT: u32 = 0x4849_0022;
pub const STAGE_KERNEL_FD_WRITE_REQ_RECV: u32 = 0x4849_0023;
pub const STAGE_KERNEL_FD_WRITE_GPIO_DONE: u32 = 0x4849_0024;
pub const STAGE_KERNEL_PROC_EXIT_RECV: u32 = 0x4849_0025;
pub const STAGE_KERNEL_RUN_SEND_BEGIN: u32 = 0x4849_0026;
pub const STAGE_KERNEL_RUN_SEND_DONE: u32 = 0x4849_0027;
pub const STAGE_ENGINE_BEGIN: u32 = 0x4849_0030;
pub const STAGE_ENGINE_FD_WRITE_BEGIN: u32 = 0x4849_0031;
pub const STAGE_ENGINE_FD_WRITE_BORROW_SENT: u32 = 0x4849_0032;
pub const STAGE_ENGINE_RUNTIME_READY_SEEN: u32 = 0x4849_0033;
pub const STAGE_ENGINE_ENDPOINT_READY: u32 = 0x4849_0034;
pub const STAGE_ENGINE_PARSE_DONE: u32 = 0x4849_0035;
pub const STAGE_ENGINE_PROC_EXIT_SENT: u32 = 0x4849_0036;
pub const STAGE_ENGINE_LOOP_CONTINUE_SENT: u32 = 0x4849_0037;
pub const STAGE_ENGINE_LOOP_BREAK_SENT: u32 = 0x4849_0038;
pub const STAGE_ENGINE_TRAP_FD_WRITE: u32 = 0x4849_0039;
pub const STAGE_ENGINE_TRAP_POLL_ONEOFF: u32 = 0x4849_003a;
pub const STAGE_ENGINE_TRAP_ENVIRON_SIZES: u32 = 0x4849_003b;
pub const STAGE_ENGINE_TRAP_ENVIRON_GET: u32 = 0x4849_003c;
pub const STAGE_ENGINE_TRAP_ARGS_SIZES: u32 = 0x4849_003d;
pub const STAGE_ENGINE_TRAP_ARGS_GET: u32 = 0x4849_003e;
pub const STAGE_ENGINE_TRAP_MEMORY_GROW: u32 = 0x4849_003f;
pub const STAGE_ENGINE_TRAP_UNSUPPORTED: u32 = 0x4849_0040;
pub const STAGE_ENGINE_RUN_RECV_BEGIN: u32 = 0x4849_0041;
pub const STAGE_ENGINE_BORROW_SEND_ERR: u32 = 0x4849_0042;
pub const STAGE_ENGINE_RUN_RECV_DONE: u32 = 0x4849_0046;
pub const STAGE_KERNEL_RUN_SEND_ERR: u32 = 0x4849_0047;
pub const STAGE_ENGINE_RUN_RECV_ERR: u32 = 0x4849_0048;
pub const STAGE_ENGINE_RUN_MISMATCH: u32 = 0x4849_0049;
pub const STAGE_KERNEL_RUN_FLOW_ERR: u32 = 0x4849_004a;
pub const STAGE_ENGINE_RESUME_ERR_TRUNCATED: u32 = 0x4849_0101;
pub const STAGE_ENGINE_RESUME_ERR_INVALID: u32 = 0x4849_0102;
pub const STAGE_ENGINE_RESUME_ERR_UNSUPPORTED: u32 = 0x4849_0103;
pub const STAGE_ENGINE_RESUME_ERR_STACK_OVERFLOW: u32 = 0x4849_0104;
pub const STAGE_ENGINE_RESUME_ERR_STACK_UNDERFLOW: u32 = 0x4849_0105;
pub const STAGE_ENGINE_RESUME_ERR_PENDING: u32 = 0x4849_0106;
pub const STAGE_ENGINE_RESUME_ERR_REPLY: u32 = 0x4849_0107;
pub const STAGE_ENGINE_RESUME_ERR_TRAP: u32 = 0x4849_0108;
pub const STAGE_ENGINE_RESUME_ERR_FUEL: u32 = 0x4849_0109;
pub const STAGE_ENGINE_RESUME_ERR_OPCODE_BASE: u32 = 0x4849_0200;

#[cfg(feature = "baker-bad-order-demo")]
pub const STAGE_BAD_ORDER_POLL_REJECTED: u32 = 0x4849_0043;
#[cfg(feature = "baker-invalid-fd-demo")]
pub const STAGE_INVALID_FD_REJECTED: u32 = 0x4849_0044;
#[cfg(feature = "baker-bad-payload-demo")]
pub const STAGE_BAD_PAYLOAD_REJECTED: u32 = 0x4849_0045;
#[cfg(feature = "baker-choreofs-bad-path-demo")]
pub const STAGE_BAD_PATH_REJECTED: u32 = 0x4849_004b;
#[cfg(feature = "baker-choreofs-bad-payload-demo")]
pub const STAGE_CHOREOFS_BAD_PAYLOAD_REJECTED: u32 = 0x4849_004c;
#[cfg(feature = "baker-choreofs-wrong-object-demo")]
pub const STAGE_WRONG_OBJECT_REJECTED: u32 = 0x4849_004d;

pub const STAGE_ENGINE_TRAP_PATH_OPEN: u32 = 0x4849_0050;
pub const STAGE_ENGINE_PATH_OPEN_BEGIN: u32 = 0x4849_0051;
pub const STAGE_ENGINE_PATH_OPEN_BORROW_SENT: u32 = 0x4849_0052;
pub const STAGE_ENGINE_PATH_OPEN_GRANT_RECV: u32 = 0x4849_0053;
pub const STAGE_ENGINE_PATH_OPEN_PATH_DECODED: u32 = 0x4849_0054;
pub const STAGE_ENGINE_PATH_OPEN_REQ_SENT: u32 = 0x4849_0055;
pub const STAGE_ENGINE_PATH_OPEN_RET_RECV: u32 = 0x4849_0056;
pub const STAGE_ENGINE_PATH_OPEN_RELEASE_SENT: u32 = 0x4849_0057;
pub const STAGE_ENGINE_PATH_OPEN_COMPLETED: u32 = 0x4849_0058;
#[cfg(any(
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
pub const STAGE_KERNEL_PATH_OPEN_BORROW_RECV: u32 = 0x4849_0060;
#[cfg(any(
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
pub const STAGE_KERNEL_PATH_OPEN_GRANT_SENT: u32 = 0x4849_0061;
#[cfg(any(
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
pub const STAGE_KERNEL_PATH_OPEN_REQ_RECV: u32 = 0x4849_0062;
#[cfg(any(
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
pub const STAGE_KERNEL_PATH_OPEN_OBJECT_OPENED: u32 = 0x4849_0063;
#[cfg(any(
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
pub const STAGE_KERNEL_PATH_OPEN_RET_SENT: u32 = 0x4849_0064;
#[cfg(any(
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo"
))]
pub const STAGE_KERNEL_PATH_OPEN_RELEASE_RECV: u32 = 0x4849_0065;
