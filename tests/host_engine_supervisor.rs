use hibana::{
    g,
    g::{Msg, Role},
    substrate::{
        SessionKit,
        binding::NoBinding,
        ids::SessionId,
        program::{RoleProgram, project},
        runtime::{Config, CounterClock},
        tap::TapEvent,
    },
};
use hibana_pico::{
    choreography::protocol::{
        BudgetExpired, BudgetExpiredMsg, BudgetRestart, BudgetRestartMsg, BudgetRun, BudgetRunMsg,
        BudgetSuspend, BudgetSuspendMsg, EngineLabelUniverse, EngineReq, EngineRet,
        LABEL_ENGINE_REQ, LABEL_ENGINE_RET,
    },
    kernel::budget::BudgetController,
    kernel::engine::wasm::{BudgetedGuestTrap, FUEL_EXHAUSTION_WASM_GUEST, TinyWasmInstance},
    kernel::resolver::{InterruptEvent, PicoInterruptResolver, ResolvedInterrupt},
    substrate::host_queue::HostQueueBackend,
    substrate::transport::SioTransport,
};

const ENGINE_LOG_VALUE: u32 = 0x4849_4241;

type TestTransport<'a> = SioTransport<&'a HostQueueBackend>;
type TestKit<'a> = SessionKit<'a, TestTransport<'a>, EngineLabelUniverse, CounterClock, 1>;

fn project_engine_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = g::seq(
        g::send::<Role<1>, Role<0>, Msg<LABEL_ENGINE_REQ, EngineReq>, 0>(),
        g::seq(
            g::send::<Role<0>, Role<1>, Msg<LABEL_ENGINE_RET, EngineRet>, 0>(),
            g::seq(
                g::send::<Role<1>, Role<0>, Msg<LABEL_ENGINE_REQ, EngineReq>, 0>(),
                g::send::<Role<0>, Role<1>, Msg<LABEL_ENGINE_RET, EngineRet>, 0>(),
            ),
        ),
    );
    let core0: RoleProgram<0> = project(&program);
    let core1: RoleProgram<1> = project(&program);
    (core0, core1)
}

fn project_budget_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = g::seq(
        g::send::<Role<0>, Role<1>, BudgetRunMsg, 0>(),
        g::seq(
            g::send::<Role<1>, Role<0>, BudgetExpiredMsg, 0>(),
            g::seq(
                g::send::<Role<0>, Role<1>, BudgetSuspendMsg, 0>(),
                g::send::<Role<0>, Role<1>, BudgetRestartMsg, 0>(),
            ),
        ),
    );
    let core0: RoleProgram<0> = project(&program);
    let core1: RoleProgram<1> = project(&program);
    (core0, core1)
}

#[test]
fn host_backend_roundtrips_engine_supervisor_requests() {
    hibana_pico::substrate::exec::run_current_task(async {
        let backend = HostQueueBackend::new();

        let clock0 = CounterClock::new();
        let mut tap0 = [TapEvent::zero(); 128];
        let mut slab0 = vec![0u8; 262_144];
        let cluster0 = TestKit::new(&clock0);
        let rv0 = cluster0
            .add_rendezvous_from_config(
                Config::new(&mut tap0, slab0.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .expect("register core0 rendezvous");

        let clock1 = CounterClock::new();
        let mut tap1 = [TapEvent::zero(); 128];
        let mut slab1 = vec![0u8; 262_144];
        let cluster1 = TestKit::new(&clock1);
        let rv1 = cluster1
            .add_rendezvous_from_config(
                Config::new(&mut tap1, slab1.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .expect("register core1 rendezvous");

        let sid = SessionId::new(23);
        let (core0_program, core1_program) = project_engine_roles();
        let mut supervisor = cluster0
            .enter(rv0, sid, &core0_program, NoBinding)
            .expect("attach supervisor endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &core1_program, NoBinding)
            .expect("attach engine endpoint");

        let request = EngineReq::LogU32(ENGINE_LOG_VALUE);
        let _ = (engine
            .flow::<Msg<LABEL_ENGINE_REQ, EngineReq>>()
            .expect("engine flow<req>")
            .send(&request))
        .await
        .expect("engine send log_u32");

        let received = (supervisor.recv::<Msg<LABEL_ENGINE_REQ, EngineReq>>())
            .await
            .expect("supervisor recv log_u32");
        assert_eq!(received, request);

        let reply = EngineRet::Logged(ENGINE_LOG_VALUE);
        let _ = (supervisor
            .flow::<Msg<LABEL_ENGINE_RET, EngineRet>>()
            .expect("supervisor flow<ret>")
            .send(&reply))
        .await
        .expect("supervisor send logged");

        let received = (engine.recv::<Msg<LABEL_ENGINE_RET, EngineRet>>())
            .await
            .expect("engine recv logged");
        assert_eq!(received, reply);

        let request = EngineReq::Yield;
        let _ = (engine
            .flow::<Msg<LABEL_ENGINE_REQ, EngineReq>>()
            .expect("engine flow<req>")
            .send(&request))
        .await
        .expect("engine send yield");

        let received = (supervisor.recv::<Msg<LABEL_ENGINE_REQ, EngineReq>>())
            .await
            .expect("supervisor recv yield");
        assert_eq!(received, request);

        let reply = EngineRet::Yielded;
        let _ = (supervisor
            .flow::<Msg<LABEL_ENGINE_RET, EngineRet>>()
            .expect("supervisor flow<ret>")
            .send(&reply))
        .await
        .expect("supervisor send yielded");

        let received = (engine.recv::<Msg<LABEL_ENGINE_RET, EngineRet>>())
            .await
            .expect("engine recv yielded");
        assert_eq!(received, reply);
    });
}

#[test]
fn budget_expiry_is_choreography_before_suspend_or_restart() {
    hibana_pico::substrate::exec::run_current_task(async {
        let backend = HostQueueBackend::new();

        let clock0 = CounterClock::new();
        let mut tap0 = [TapEvent::zero(); 128];
        let mut slab0 = vec![0u8; 262_144];
        let cluster0 = TestKit::new(&clock0);
        let rv0 = cluster0
            .add_rendezvous_from_config(
                Config::new(&mut tap0, slab0.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .expect("register core0 rendezvous");

        let clock1 = CounterClock::new();
        let mut tap1 = [TapEvent::zero(); 128];
        let mut slab1 = vec![0u8; 262_144];
        let cluster1 = TestKit::new(&clock1);
        let rv1 = cluster1
            .add_rendezvous_from_config(
                Config::new(&mut tap1, slab1.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .expect("register core1 rendezvous");

        let sid = SessionId::new(24);
        let (core0_program, core1_program) = project_budget_roles();
        let mut supervisor = cluster0
            .enter(rv0, sid, &core0_program, NoBinding)
            .expect("attach supervisor endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &core1_program, NoBinding)
            .expect("attach engine endpoint");

        let mut budget = BudgetController::new();
        let run = BudgetRun::new(7, 3, 1000, 123_456);
        let run = budget.start(run).expect("start current budget run");
        let _ = (supervisor
            .flow::<BudgetRunMsg>()
            .expect("supervisor flow<budget run>")
            .send(&run))
        .await
        .expect("supervisor sends budget run");

        let received_run = (engine.recv::<BudgetRunMsg>())
            .await
            .expect("engine receives budget run");
        assert_eq!(received_run, run);

        assert!(
            supervisor.flow::<BudgetSuspendMsg>().is_err(),
            "kernel cannot suspend until budget expiry reaches the projected phase"
        );

        let mut resolver: PicoInterruptResolver<1, 1, 1> = PicoInterruptResolver::new();
        resolver
            .push_irq(InterruptEvent::BudgetTimerExpired {
                run_id: run.run_id(),
                generation: run.generation(),
            })
            .expect("record budget timer expiry");
        let admitted = match resolver.resolve_next().expect("resolve budget expiry") {
            Some(ResolvedInterrupt::BudgetExpired(expired)) => expired,
            other => panic!("expected budget expiry ready fact, got {other:?}"),
        };
        assert_eq!(admitted, BudgetExpired::new(run.run_id(), run.generation()));
        let admitted = budget
            .admit_expiry(admitted)
            .expect("budget controller admits current expiry");

        let _ = (engine
            .flow::<BudgetExpiredMsg>()
            .expect("engine flow<budget expired>")
            .send(&admitted))
        .await
        .expect("engine sends budget expired");

        let received_expired = (supervisor.recv::<BudgetExpiredMsg>())
            .await
            .expect("supervisor receives budget expired");
        assert_eq!(received_expired, admitted);

        let suspend = budget
            .suspend_after_expiry()
            .expect("budget controller allows suspend after expiry");
        assert_eq!(suspend, BudgetSuspend::new(run.run_id(), run.generation()));
        let _ = (supervisor
            .flow::<BudgetSuspendMsg>()
            .expect("supervisor flow<budget suspend>")
            .send(&suspend))
        .await
        .expect("supervisor sends budget suspend");

        let received_suspend = (engine.recv::<BudgetSuspendMsg>())
            .await
            .expect("engine receives budget suspend");
        assert_eq!(received_suspend, suspend);

        let restart = budget
            .restart_after_suspend(BudgetRestart::new(
                run.run_id(),
                run.generation() + 1,
                500,
                124_000,
            ))
            .expect("budget controller allows restart after suspend");
        let _ = (supervisor
            .flow::<BudgetRestartMsg>()
            .expect("supervisor flow<budget restart>")
            .send(&restart))
        .await
        .expect("supervisor sends budget restart");

        let received_restart = (engine.recv::<BudgetRestartMsg>())
            .await
            .expect("engine receives budget restart");
        assert_eq!(received_restart, restart);
    });
}

#[test]
fn wasm_fuel_exhaustion_reaches_kernel_as_budget_expired_choreography() {
    hibana_pico::substrate::exec::run_current_task(async {
        let backend = HostQueueBackend::new();

        let clock0 = CounterClock::new();
        let mut tap0 = [TapEvent::zero(); 128];
        let mut slab0 = vec![0u8; 262_144];
        let cluster0 = TestKit::new(&clock0);
        let rv0 = cluster0
            .add_rendezvous_from_config(
                Config::new(&mut tap0, slab0.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .expect("register core0 rendezvous");

        let clock1 = CounterClock::new();
        let mut tap1 = [TapEvent::zero(); 128];
        let mut slab1 = vec![0u8; 262_144];
        let cluster1 = TestKit::new(&clock1);
        let rv1 = cluster1
            .add_rendezvous_from_config(
                Config::new(&mut tap1, slab1.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .expect("register core1 rendezvous");

        let sid = SessionId::new(25);
        let (core0_program, core1_program) = project_budget_roles();
        let mut kernel = cluster0
            .enter(rv0, sid, &core0_program, NoBinding)
            .expect("attach kernel endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &core1_program, NoBinding)
            .expect("attach engine endpoint");

        let mut budget = BudgetController::new();
        let run = budget
            .start(BudgetRun::new(9, 1, 8, 64))
            .expect("start budget run");
        let _ = (kernel
            .flow::<BudgetRunMsg>()
            .expect("kernel flow<budget run>")
            .send(&run))
        .await
        .expect("kernel sends budget run");

        let received_run = (engine.recv::<BudgetRunMsg>())
            .await
            .expect("engine receives budget run");
        let mut guest =
            TinyWasmInstance::new(FUEL_EXHAUSTION_WASM_GUEST).expect("instantiate fuel guest");
        let expired = match guest
            .resume_with_budget(received_run)
            .expect("budgeted wasm resume")
        {
            BudgetedGuestTrap::BudgetExpired(expired) => expired,
            other => panic!("expected budget expiry, got {other:?}"),
        };
        assert_eq!(
            expired,
            BudgetExpired::new(received_run.run_id(), received_run.generation())
        );

        let _ = (engine
            .flow::<BudgetExpiredMsg>()
            .expect("engine flow<budget expired>")
            .send(&expired))
        .await
        .expect("engine sends budget expired");

        let received_expired = (kernel.recv::<BudgetExpiredMsg>())
            .await
            .expect("kernel receives budget expired");
        let admitted = budget
            .admit_expiry(received_expired)
            .expect("kernel admits current budget expiry");
        assert_eq!(admitted, expired);

        let suspend = budget
            .suspend_after_expiry()
            .expect("kernel suspends after budget expiry");
        let _ = (kernel
            .flow::<BudgetSuspendMsg>()
            .expect("kernel flow<budget suspend>")
            .send(&suspend))
        .await
        .expect("kernel sends budget suspend");
        assert_eq!(
            engine
                .recv::<BudgetSuspendMsg>()
                .await
                .expect("engine receives budget suspend"),
            suspend
        );

        let restart = budget
            .restart_after_suspend(BudgetRestart::new(
                received_run.run_id(),
                received_run.generation() + 1,
                8,
                128,
            ))
            .expect("kernel restarts after suspend");
        let _ = (kernel
            .flow::<BudgetRestartMsg>()
            .expect("kernel flow<budget restart>")
            .send(&restart))
        .await
        .expect("kernel sends budget restart");
        assert_eq!(
            engine
                .recv::<BudgetRestartMsg>()
                .await
                .expect("engine receives budget restart"),
            restart
        );
    });
}
