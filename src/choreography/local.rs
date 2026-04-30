//! Local syscall/device choreography entry point.
//!
//! Read this module first when checking the single-node story. Each function
//! names one local protocol, shows the global order with `hibana::g`, and
//! returns the two projected role programs used by host tests and firmware
//! demos. The matching localside code should drive the returned programs with
//! only `flow().send()`, `recv()`, `offer()`, or `decode()`.

use hibana::{
    g,
    g::{Msg, Role},
    substrate::program::{RoleProgram, project},
};

use crate::choreography::protocol::{
    BakerTrafficLoopBreakControl, BakerTrafficLoopContinueControl, BudgetRunMsg, EngineReq,
    EngineRet, GpioSet, LABEL_GPIO_SET, LABEL_GPIO_SET_DONE, LABEL_MEM_BORROW_READ,
    LABEL_MEM_BORROW_WRITE, LABEL_MEM_COMMIT, LABEL_MEM_FENCE, LABEL_MEM_RELEASE,
    LABEL_TIMER_SLEEP_DONE, LABEL_TIMER_SLEEP_UNTIL, LABEL_UART_WRITE, LABEL_UART_WRITE_RET,
    LABEL_WASI_CLOCK_RES_GET, LABEL_WASI_CLOCK_RES_GET_RET, LABEL_WASI_FD_CLOSE,
    LABEL_WASI_FD_CLOSE_RET, LABEL_WASI_FD_FDSTAT_GET, LABEL_WASI_FD_FDSTAT_GET_RET,
    LABEL_WASI_FD_READ, LABEL_WASI_FD_READ_RET, LABEL_WASI_FD_WRITE, LABEL_WASI_FD_WRITE_RET,
    LABEL_WASI_POLL_ONEOFF, LABEL_WASI_POLL_ONEOFF_RET, LABEL_WASI_PROC_EXIT,
    LABEL_WASIP1_CLOCK_NOW, LABEL_WASIP1_CLOCK_NOW_RET, LABEL_WASIP1_EXIT,
    LABEL_WASIP1_RANDOM_SEED, LABEL_WASIP1_RANDOM_SEED_RET, LABEL_WASIP1_STDERR,
    LABEL_WASIP1_STDERR_RET, LABEL_WASIP1_STDIN, LABEL_WASIP1_STDIN_RET, LABEL_WASIP1_STDOUT,
    LABEL_WASIP1_STDOUT_RET, LABEL_YIELD_REQ, LABEL_YIELD_RET, MemBorrow, MemCommit, MemFence,
    MemReadGrantControl, MemRelease, MemWriteGrantControl, POLICY_BAKER_TRAFFIC_LOOP,
    TimerSleepDone, TimerSleepUntil, UartWrite, UartWriteDone,
};

macro_rules! seq_chain {
    ($head:expr, $($tail:expr),+ $(,)?) => {
        g::seq($head, seq_chain!($($tail),+))
    };
    ($last:expr $(,)?) => {
        $last
    };
}

macro_rules! baker_led_fd_write_cycle {
    () => {
        seq_chain!(
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
            g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_FD_WRITE, EngineReq>, 1>(),
            g::send::<Role<0>, Role<2>, Msg<LABEL_GPIO_SET, GpioSet>, 1>(),
            g::send::<Role<2>, Role<0>, Msg<LABEL_GPIO_SET_DONE, GpioSet>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_FD_WRITE_RET, EngineRet>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
        )
    };
}

macro_rules! baker_led_poll_cycle {
    () => {
        seq_chain!(
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_POLL_ONEOFF, EngineReq>, 1>(),
            g::send::<Role<0>, Role<3>, Msg<LABEL_TIMER_SLEEP_UNTIL, TimerSleepUntil>, 1>(),
            g::send::<Role<3>, Role<0>, Msg<LABEL_TIMER_SLEEP_DONE, TimerSleepDone>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_POLL_ONEOFF_RET, EngineRet>, 1>(),
        )
    };
}

macro_rules! baker_led_fd_write_two_cycles_program {
    () => {
        g::seq(baker_led_fd_write_cycle!(), baker_led_fd_write_cycle!())
    };
}

macro_rules! baker_led_traffic_light_step_program {
    () => {
        g::seq(baker_led_fd_write_cycle!(), baker_led_poll_cycle!())
    };
}

macro_rules! baker_led_blink_program {
    () => {{
        let continue_arm = g::send::<Role<1>, Role<1>, BakerTrafficLoopContinueControl, 1>()
            .policy::<POLICY_BAKER_TRAFFIC_LOOP>();
        let break_arm = g::send::<Role<1>, Role<1>, BakerTrafficLoopBreakControl, 1>()
            .policy::<POLICY_BAKER_TRAFFIC_LOOP>();
        // One app activation starts with BudgetRun. The hibana loop is a route:
        // Continue enters exactly one WASI fd_write + poll_oneoff body, while
        // Break carries the final proc_exit. The Engine role owns the control
        // message because only the guest can decide whether it has another
        // syscall step or has returned.
        seq_chain!(
            g::send::<Role<0>, Role<1>, BudgetRunMsg, 1>(),
            g::route(
                g::seq(continue_arm, baker_led_traffic_light_step_program!()),
                g::seq(
                    break_arm,
                    g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_PROC_EXIT, EngineReq>, 1>(),
                )
            ),
        )
    }};
}

pub const BAKER_LED_FD_WRITE_KERNEL_PROGRAM: RoleProgram<0> =
    project(&baker_led_fd_write_two_cycles_program!());
pub const BAKER_LED_FD_WRITE_ENGINE_PROGRAM: RoleProgram<1> =
    project(&baker_led_fd_write_two_cycles_program!());
pub const BAKER_LED_FD_WRITE_GPIO_PROGRAM: RoleProgram<2> =
    project(&baker_led_fd_write_two_cycles_program!());

pub const BAKER_LED_BLINK_KERNEL_PROGRAM: RoleProgram<0> = project(&baker_led_blink_program!());
pub const BAKER_LED_BLINK_ENGINE_PROGRAM: RoleProgram<1> = project(&baker_led_blink_program!());
pub const BAKER_LED_BLINK_GPIO_PROGRAM: RoleProgram<2> = project(&baker_led_blink_program!());
pub const BAKER_LED_BLINK_TIMER_PROGRAM: RoleProgram<3> = project(&baker_led_blink_program!());

/// WASI stdout over a read lease:
/// borrow-read -> grant -> fd/stdout write -> write-ret -> release.
pub fn wasip1_stdout_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDOUT, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
    );
    (project(&program), project(&program))
}

/// WASI stdout through the local UART device role. This is the fully composed
/// form of stdout: Engine owns the lease/syscall phases, Kernel owns fd
/// resolution, and Uart owns the device write phase.
pub fn wasip1_stdout_uart_roles() -> (RoleProgram<0>, RoleProgram<1>, RoleProgram<2>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDOUT, EngineReq>, 1>(),
        g::send::<Role<0>, Role<2>, Msg<LABEL_UART_WRITE, UartWrite>, 1>(),
        g::send::<Role<2>, Role<0>, Msg<LABEL_UART_WRITE_RET, UartWriteDone>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
    );
    (project(&program), project(&program), project(&program))
}

/// WASI stderr over a read lease.
pub fn wasip1_stderr_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDERR, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDERR_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
    );
    (project(&program), project(&program))
}

/// WASI stdin over a write lease:
/// borrow-write -> grant -> fd/stdin read -> read-ret -> commit -> release.
pub fn wasip1_stdin_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_WRITE, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemWriteGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDIN, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDIN_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_COMMIT, MemCommit>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
    );
    (project(&program), project(&program))
}

/// Synchronous WASI clock syscall. This is not interrupt-driven sleep; it is a
/// request/reply clock query.
pub fn wasip1_clock_now_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_CLOCK_NOW, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_CLOCK_NOW_RET, EngineRet>, 1>(),
    );
    (project(&program), project(&program))
}

/// WASI random seed request/reply.
pub fn wasip1_random_seed_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_RANDOM_SEED, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_RANDOM_SEED_RET, EngineRet>, 1>(),
    );
    (project(&program), project(&program))
}

/// WASI proc_exit is one-way from Engine to Kernel.
pub fn wasip1_exit_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_EXIT, EngineReq>, 1>();
    (project(&program), project(&program))
}

/// WASI sched_yield maps to the existing Engine yield choreography. The WASI
/// name is an Engine/import-trampoline concern; the Kernel role sees only the
/// typed yield request/return phase.
pub fn wasip1_sched_yield_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_YIELD_REQ, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_YIELD_RET, EngineRet>, 1>(),
    );
    (project(&program), project(&program))
}

/// WASI clock_res_get as a typed clock resolution request/reply.
pub fn wasi_clock_res_get_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_CLOCK_RES_GET, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_CLOCK_RES_GET_RET, EngineRet>, 1>(),
    );
    (project(&program), project(&program))
}

/// WASI fd_read followed by fdstat and close. The fd_read result uses a write
/// lease and must be committed before release.
pub fn wasi_fd_read_stat_close_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_WRITE, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemWriteGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_FD_READ, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_FD_READ_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_COMMIT, MemCommit>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_FD_FDSTAT_GET, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_FD_FDSTAT_GET_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_FD_CLOSE, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_FD_CLOSE_RET, EngineRet>, 1>(),
    );
    (project(&program), project(&program))
}

/// WASI poll_oneoff as a synchronous request/reply over choreography. Timer
/// sleep uses `timer_sleep_roles` instead.
pub fn wasi_poll_oneoff_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_POLL_ONEOFF, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_POLL_ONEOFF_RET, EngineRet>, 1>(),
    );
    (project(&program), project(&program))
}

/// Interrupt-admitted timer sleep:
/// Engine requests sleep, resolver admits a due timer fact, then Kernel sends
/// the typed completion only when this projected phase is open.
pub fn timer_sleep_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_TIMER_SLEEP_UNTIL, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_TIMER_SLEEP_DONE, EngineRet>, 1>(),
    );
    (project(&program), project(&program))
}

/// Local UART device protocol:
/// Kernel sends bounded bytes to the UART role and waits for an explicit device
/// acknowledgement before returning to the app-visible syscall phase.
pub fn uart_write_roles() -> (RoleProgram<0>, RoleProgram<2>) {
    let program = seq_chain!(
        g::send::<Role<0>, Role<2>, Msg<LABEL_UART_WRITE, UartWrite>, 1>(),
        g::send::<Role<2>, Role<0>, Msg<LABEL_UART_WRITE_RET, UartWriteDone>, 1>(),
    );
    (project(&program), project(&program))
}

/// Baker link LED fd_write demo. The guest writes ASCII `1` then `0` to fd 3;
/// each write is gated by a read lease and acknowledged by the GPIO device role
/// before Kernel returns to Engine.
pub fn baker_led_fd_write_two_cycles_roles() -> (RoleProgram<0>, RoleProgram<1>, RoleProgram<2>) {
    (
        BAKER_LED_FD_WRITE_KERNEL_PROGRAM,
        BAKER_LED_FD_WRITE_ENGINE_PROGRAM,
        BAKER_LED_FD_WRITE_GPIO_PROGRAM,
    )
}

/// Baker link LED traffic-light demo. Kernel sends one `BudgetRunMsg`; after
/// that the Engine-owned hibana loop route decides whether the WASI app has
/// another fd_write/poll body (`LoopContinue`) or has returned (`LoopBreak +
/// proc_exit`). The guest-visible body uses fds 3, 4, and 5. Every visible
/// transition is a WASI `fd_write`, every wait is a WASI `poll_oneoff` admitted
/// by the Timer resolver, and Kernel never restarts the app unless a separate
/// lifecycle policy says so.
pub fn baker_led_blink_roles() -> (
    RoleProgram<0>,
    RoleProgram<1>,
    RoleProgram<2>,
    RoleProgram<3>,
) {
    (
        BAKER_LED_BLINK_KERNEL_PROGRAM,
        BAKER_LED_BLINK_ENGINE_PROGRAM,
        BAKER_LED_BLINK_GPIO_PROGRAM,
        BAKER_LED_BLINK_TIMER_PROGRAM,
    )
}

/// Artifact smoke for `memory.grow`: the grow is admitted by a memory fence,
/// old leases are invalidated, and the next syscall must borrow under the new
/// epoch before stdout can proceed.
pub fn memory_grow_stdout_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_FENCE, MemFence>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDOUT, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
    );
    (project(&program), project(&program))
}

/// Clock followed by stdout. Used by Rust-built artifact tests to show syscall
/// order plus pointer-backed stdout lease in one local choreography.
pub fn wasip1_clock_stdout_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_CLOCK_NOW, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_CLOCK_NOW_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDOUT, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
    );
    (project(&program), project(&program))
}

/// Stdin followed by stdout. Used by artifact tests to show write lease commit
/// and later read lease in one local choreography.
pub fn wasip1_stdin_stdout_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = seq_chain!(
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_WRITE, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemWriteGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDIN, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDIN_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_COMMIT, MemCommit>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
        g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_WASIP1_STDOUT, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_WASIP1_STDOUT_RET, EngineRet>, 1>(),
        g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
    );
    (project(&program), project(&program))
}
