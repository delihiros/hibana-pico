//! Baker Link Dev Rev1 LED proof choreography.
//!
//! This module is intentionally separate from `choreography::local`: Baker Link
//! is the public RP2040 hardware proof, not the generic syscall/device protocol
//! vocabulary. The roles are:
//!
//! - Role 0: Kernel
//! - Role 1: Engine / WASI P1 app
//! - Role 2: GPIO device
//! - Role 3: Timer device

use hibana::{
    g,
    g::{Msg, Role},
    substrate::{
        cap::{
            GenericCapToken,
            advanced::{LoopBreakKind, LoopContinueKind},
        },
        program::{RoleProgram, project},
    },
};

use crate::choreography::protocol::{
    BudgetRunMsg, EngineReq, EngineRet, GpioSet, LABEL_GPIO_SET, LABEL_GPIO_SET_DONE,
    LABEL_MEM_BORROW_READ, LABEL_MEM_RELEASE, LABEL_TIMER_SLEEP_DONE, LABEL_TIMER_SLEEP_UNTIL,
    LABEL_WASI_FD_WRITE, LABEL_WASI_FD_WRITE_RET, LABEL_WASI_PATH_OPEN, LABEL_WASI_PATH_OPEN_RET,
    LABEL_WASI_POLL_ONEOFF, LABEL_WASI_POLL_ONEOFF_RET, LABEL_WASI_PROC_EXIT, MemBorrow,
    MemReadGrantControl, MemRelease, TimerSleepDone, TimerSleepUntil,
};

pub const LABEL_BAKER_TRAFFIC_LOOP_CONTINUE: u8 = 120;
pub const LABEL_BAKER_TRAFFIC_LOOP_BREAK: u8 = 121;
pub const POLICY_BAKER_TRAFFIC_LOOP: u16 = 120;

pub type BakerTrafficLoopContinueControl =
    Msg<LABEL_BAKER_TRAFFIC_LOOP_CONTINUE, GenericCapToken<LoopContinueKind>, LoopContinueKind>;
pub type BakerTrafficLoopBreakControl =
    Msg<LABEL_BAKER_TRAFFIC_LOOP_BREAK, GenericCapToken<LoopBreakKind>, LoopBreakKind>;

macro_rules! seq_chain {
    ($head:expr, $($tail:expr),+ $(,)?) => {
        g::seq($head, seq_chain!($($tail),+))
    };
    ($last:expr $(,)?) => {
        $last
    };
}

macro_rules! fd_write_cycle {
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

macro_rules! path_open_cycle {
    () => {
        seq_chain!(
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_BORROW_READ, MemBorrow>, 1>(),
            g::send::<Role<0>, Role<1>, MemReadGrantControl, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_PATH_OPEN, EngineReq>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_PATH_OPEN_RET, EngineRet>, 1>(),
            g::send::<Role<1>, Role<0>, Msg<LABEL_MEM_RELEASE, MemRelease>, 1>(),
        )
    };
}

macro_rules! poll_cycle {
    () => {
        seq_chain!(
            g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_POLL_ONEOFF, EngineReq>, 1>(),
            g::send::<Role<0>, Role<3>, Msg<LABEL_TIMER_SLEEP_UNTIL, TimerSleepUntil>, 1>(),
            g::send::<Role<3>, Role<0>, Msg<LABEL_TIMER_SLEEP_DONE, TimerSleepDone>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<LABEL_WASI_POLL_ONEOFF_RET, EngineRet>, 1>(),
        )
    };
}

macro_rules! fd_write_two_cycles_program {
    () => {
        g::seq(fd_write_cycle!(), fd_write_cycle!())
    };
}

macro_rules! traffic_light_step_program {
    () => {
        g::seq(fd_write_cycle!(), poll_cycle!())
    };
}

macro_rules! traffic_light_program {
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
                g::seq(continue_arm, traffic_light_step_program!()),
                g::seq(
                    break_arm,
                    g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_PROC_EXIT, EngineReq>, 1>(),
                )
            ),
        )
    }};
}

macro_rules! choreofs_traffic_light_program {
    () => {{
        let continue_arm = g::send::<Role<1>, Role<1>, BakerTrafficLoopContinueControl, 1>()
            .policy::<POLICY_BAKER_TRAFFIC_LOOP>();
        let break_arm = g::send::<Role<1>, Role<1>, BakerTrafficLoopBreakControl, 1>()
            .policy::<POLICY_BAKER_TRAFFIC_LOOP>();
        seq_chain!(
            g::send::<Role<0>, Role<1>, BudgetRunMsg, 1>(),
            path_open_cycle!(),
            path_open_cycle!(),
            path_open_cycle!(),
            g::route(
                g::seq(continue_arm, traffic_light_step_program!()),
                g::seq(
                    break_arm,
                    g::send::<Role<1>, Role<0>, Msg<LABEL_WASI_PROC_EXIT, EngineReq>, 1>(),
                )
            ),
        )
    }};
}

pub const FD_WRITE_KERNEL_PROGRAM: RoleProgram<0> = project(&fd_write_two_cycles_program!());
pub const FD_WRITE_ENGINE_PROGRAM: RoleProgram<1> = project(&fd_write_two_cycles_program!());
pub const FD_WRITE_GPIO_PROGRAM: RoleProgram<2> = project(&fd_write_two_cycles_program!());

pub const TRAFFIC_LIGHT_KERNEL_PROGRAM: RoleProgram<0> = project(&traffic_light_program!());
pub const TRAFFIC_LIGHT_ENGINE_PROGRAM: RoleProgram<1> = project(&traffic_light_program!());
pub const TRAFFIC_LIGHT_GPIO_PROGRAM: RoleProgram<2> = project(&traffic_light_program!());
pub const TRAFFIC_LIGHT_TIMER_PROGRAM: RoleProgram<3> = project(&traffic_light_program!());

pub const CHOREOFS_TRAFFIC_LIGHT_KERNEL_PROGRAM: RoleProgram<0> =
    project(&choreofs_traffic_light_program!());
pub const CHOREOFS_TRAFFIC_LIGHT_ENGINE_PROGRAM: RoleProgram<1> =
    project(&choreofs_traffic_light_program!());
pub const CHOREOFS_TRAFFIC_LIGHT_GPIO_PROGRAM: RoleProgram<2> =
    project(&choreofs_traffic_light_program!());
pub const CHOREOFS_TRAFFIC_LIGHT_TIMER_PROGRAM: RoleProgram<3> =
    project(&choreofs_traffic_light_program!());

/// Baker Link LED fd_write proof. The guest writes ASCII `1` then `0` to fd 3;
/// each write is gated by a read lease and acknowledged by the GPIO device role
/// before Kernel returns to Engine.
pub fn fd_write_two_cycles_roles() -> (RoleProgram<0>, RoleProgram<1>, RoleProgram<2>) {
    (
        FD_WRITE_KERNEL_PROGRAM,
        FD_WRITE_ENGINE_PROGRAM,
        FD_WRITE_GPIO_PROGRAM,
    )
}

/// Baker Link LED traffic-light proof. Kernel sends one `BudgetRunMsg`; after
/// that the Engine-owned hibana loop route decides whether the WASI app has
/// another fd_write/poll body (`LoopContinue`) or has returned (`LoopBreak +
/// proc_exit`). Every wait is a WASI `poll_oneoff` admitted by the Timer
/// resolver.
pub fn traffic_light_roles() -> (
    RoleProgram<0>,
    RoleProgram<1>,
    RoleProgram<2>,
    RoleProgram<3>,
) {
    (
        TRAFFIC_LIGHT_KERNEL_PROGRAM,
        TRAFFIC_LIGHT_ENGINE_PROGRAM,
        TRAFFIC_LIGHT_GPIO_PROGRAM,
        TRAFFIC_LIGHT_TIMER_PROGRAM,
    )
}

/// Baker Link ChoreoFS LED proof. The guest first opens LED resource paths
/// through WASI `path_open`, so the GPIO fds are minted from ChoreoFS object
/// identities before the same fd_write/poll loop begins.
pub fn choreofs_traffic_light_roles() -> (
    RoleProgram<0>,
    RoleProgram<1>,
    RoleProgram<2>,
    RoleProgram<3>,
) {
    (
        CHOREOFS_TRAFFIC_LIGHT_KERNEL_PROGRAM,
        CHOREOFS_TRAFFIC_LIGHT_ENGINE_PROGRAM,
        CHOREOFS_TRAFFIC_LIGHT_GPIO_PROGRAM,
        CHOREOFS_TRAFFIC_LIGHT_TIMER_PROGRAM,
    )
}
