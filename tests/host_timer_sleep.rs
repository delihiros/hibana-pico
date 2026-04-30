use hibana::{
    g::Msg,
    substrate::{
        SessionKit,
        binding::NoBinding,
        ids::SessionId,
        runtime::{Config, CounterClock},
        tap::TapEvent,
    },
};
use hibana_pico::{
    choreography::local::timer_sleep_roles,
    choreography::protocol::{
        EngineLabelUniverse, EngineReq, EngineRet, LABEL_TIMER_SLEEP_DONE, LABEL_TIMER_SLEEP_UNTIL,
        TimerSleepDone, TimerSleepUntil,
    },
    kernel::engine::wasm::{GuestTrap, SLEEP_WASM_GUEST, TinyWasmInstance},
    kernel::resolver::{InterruptEvent, PicoInterruptResolver, ResolvedInterrupt},
    substrate::host_queue::HostQueueBackend,
    substrate::transport::SioTransport,
};

type TestTransport<'a> = SioTransport<&'a HostQueueBackend>;
type TestKit<'a> = SessionKit<'a, TestTransport<'a>, EngineLabelUniverse, CounterClock, 1>;

#[test]
fn host_backend_timer_sleep_request_round_trips_when_due() {
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
            .expect("register supervisor rendezvous");

        let clock1 = CounterClock::new();
        let mut tap1 = [TapEvent::zero(); 128];
        let mut slab1 = vec![0u8; 262_144];
        let cluster1 = TestKit::new(&clock1);
        let rv1 = cluster1
            .add_rendezvous_from_config(
                Config::new(&mut tap1, slab1.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .expect("register engine rendezvous");

        let sid = SessionId::new(71);
        let (supervisor_program, engine_program) = timer_sleep_roles();
        let mut supervisor = cluster0
            .enter(rv0, sid, &supervisor_program, NoBinding)
            .expect("attach supervisor endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &engine_program, NoBinding)
            .expect("attach engine endpoint");

        let mut guest = TinyWasmInstance::new(SLEEP_WASM_GUEST).expect("instantiate sleep guest");
        let GuestTrap::HostCall(EngineReq::TimerSleepUntil(sleep)) =
            guest.resume().expect("resume to sleep")
        else {
            panic!("expected sleep host call");
        };
        assert_eq!(sleep, TimerSleepUntil::new(42));

        let request = EngineReq::TimerSleepUntil(sleep);
        (engine
            .flow::<Msg<LABEL_TIMER_SLEEP_UNTIL, EngineReq>>()
            .expect("engine flow<timer sleep>")
            .send(&request))
        .await
        .expect("engine send timer sleep");

        let received = (supervisor.recv::<Msg<LABEL_TIMER_SLEEP_UNTIL, EngineReq>>())
            .await
            .expect("supervisor recv timer sleep");
        assert_eq!(received, request);
        let EngineReq::TimerSleepUntil(received_sleep) = received else {
            panic!("expected timer sleep request");
        };

        let mut resolver: PicoInterruptResolver<2, 4, 1> = PicoInterruptResolver::new();
        resolver
            .request_timer_sleep(received_sleep)
            .expect("register sleep with resolver");
        resolver
            .push_irq(InterruptEvent::TimerTick { tick: 41 })
            .expect("record early timer IRQ");
        assert_eq!(resolver.resolve_next(), Ok(None));
        resolver
            .push_irq(InterruptEvent::TimerTick { tick: 42 })
            .expect("record due timer IRQ");
        let Some(ResolvedInterrupt::TimerSleepDone(done)) =
            resolver.resolve_next().expect("resolve timer IRQ")
        else {
            panic!("expected timer sleep completion from resolver");
        };
        assert_eq!(done, TimerSleepDone::new(42));

        let reply = EngineRet::TimerSleepDone(done);
        (supervisor
            .flow::<Msg<LABEL_TIMER_SLEEP_DONE, EngineRet>>()
            .expect("supervisor flow<timer sleep done>")
            .send(&reply))
        .await
        .expect("supervisor send timer sleep done");

        let received_reply = (engine.recv::<Msg<LABEL_TIMER_SLEEP_DONE, EngineRet>>())
            .await
            .expect("engine recv timer sleep done");
        assert_eq!(received_reply, reply);
        guest
            .complete_host_call(received_reply)
            .expect("complete sleep");
        assert_eq!(
            guest.resume().expect("resume to yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
    });
}
