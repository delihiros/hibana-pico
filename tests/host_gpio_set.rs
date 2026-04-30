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
        EngineLabelUniverse, EngineReq, EngineRet, GpioEdge, GpioWait, GpioWaitMsg, GpioWaitRetMsg,
        LABEL_GPIO_SET, LABEL_GPIO_SET_DONE,
    },
    kernel::device::gpio::GpioStateTable,
    kernel::engine::wasm::{GPIO_WASM_GUEST, GuestTrap, TinyWasmInstance},
    kernel::resolver::{InterruptEvent, PicoInterruptResolver, ResolvedInterrupt},
    substrate::host_queue::HostQueueBackend,
    substrate::transport::SioTransport,
};

type TestTransport<'a> = SioTransport<&'a HostQueueBackend>;
type TestKit<'a> = SessionKit<'a, TestTransport<'a>, EngineLabelUniverse, CounterClock, 1>;

fn project_gpio_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = g::seq(
        g::send::<Role<1>, Role<0>, Msg<LABEL_GPIO_SET, EngineReq>, 1>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_GPIO_SET_DONE, EngineRet>, 1>(),
    );
    let supervisor: RoleProgram<0> = project(&program);
    let engine: RoleProgram<1> = project(&program);
    (supervisor, engine)
}

fn project_gpio_wait_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let program = g::seq(
        g::send::<Role<1>, Role<0>, GpioWaitMsg, 4>(),
        g::send::<Role<0>, Role<1>, GpioWaitRetMsg, 4>(),
    );
    let kernel: RoleProgram<0> = project(&program);
    let engine: RoleProgram<1> = project(&program);
    (kernel, engine)
}

#[test]
fn host_backend_gpio_set_from_wasm_guest_toggles_pin() {
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

        let sid = SessionId::new(72);
        let (supervisor_program, engine_program) = project_gpio_roles();
        let mut supervisor = cluster0
            .enter(rv0, sid, &supervisor_program, NoBinding)
            .expect("attach supervisor endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &engine_program, NoBinding)
            .expect("attach engine endpoint");

        let mut guest = TinyWasmInstance::new(GPIO_WASM_GUEST).expect("instantiate gpio guest");
        let GuestTrap::HostCall(EngineReq::GpioSet(set)) =
            guest.resume().expect("resume to gpio set")
        else {
            panic!("expected gpio set host call");
        };

        let request = EngineReq::GpioSet(set);
        (engine
            .flow::<Msg<LABEL_GPIO_SET, EngineReq>>()
            .expect("engine flow<gpio set>")
            .send(&request))
        .await
        .expect("engine send gpio set");

        let received = (supervisor.recv::<Msg<LABEL_GPIO_SET, EngineReq>>())
            .await
            .expect("supervisor recv gpio set");
        assert_eq!(received, request);
        let EngineReq::GpioSet(received_set) = received else {
            panic!("expected gpio set request");
        };

        let mut pins: GpioStateTable<32> = GpioStateTable::new();
        let applied = pins.apply(received_set).expect("apply gpio set");
        assert_eq!(pins.level(25), Ok(true));

        let reply = EngineRet::GpioSetDone(applied);
        (supervisor
            .flow::<Msg<LABEL_GPIO_SET_DONE, EngineRet>>()
            .expect("supervisor flow<gpio set done>")
            .send(&reply))
        .await
        .expect("supervisor send gpio set done");

        let received_reply = (engine.recv::<Msg<LABEL_GPIO_SET_DONE, EngineRet>>())
            .await
            .expect("engine recv gpio done");
        assert_eq!(received_reply, reply);
        guest
            .complete_host_call(received_reply)
            .expect("complete gpio set");
        assert_eq!(
            guest.resume().expect("resume to yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
    });
}

#[test]
fn host_backend_gpio_wait_completes_only_through_resolver_admitted_fact() {
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
            .expect("register kernel rendezvous");

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

        let sid = SessionId::new(73);
        let (kernel_program, engine_program) = project_gpio_wait_roles();
        let mut kernel = cluster0
            .enter(rv0, sid, &kernel_program, NoBinding)
            .expect("attach kernel endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &engine_program, NoBinding)
            .expect("attach engine endpoint");

        let wait = GpioWait::new(60, 7, 4, 2);
        (engine
            .flow::<GpioWaitMsg>()
            .expect("engine flow<gpio wait>")
            .send(&wait))
        .await
        .expect("engine sends gpio wait");

        let received_wait = (kernel.recv::<GpioWaitMsg>())
            .await
            .expect("kernel receives gpio wait");
        assert_eq!(received_wait, wait);

        let mut resolver: PicoInterruptResolver<1, 2, 1> = PicoInterruptResolver::new();
        resolver
            .push_irq(InterruptEvent::GpioEdge {
                pin: wait.pin(),
                high: true,
            })
            .expect("queue unsolicited edge");
        assert_eq!(
            resolver.resolve_next(),
            Err(hibana_pico::kernel::resolver::ResolverError::UnsolicitedGpioEdge)
        );

        resolver
            .request_gpio_wait(received_wait)
            .expect("register gpio wait");
        resolver
            .push_irq(InterruptEvent::GpioEdge {
                pin: wait.pin(),
                high: true,
            })
            .expect("queue subscribed edge");
        let edge = match resolver.resolve_next().expect("resolve gpio edge") {
            Some(ResolvedInterrupt::GpioWaitSatisfied(edge)) => edge,
            other => panic!("expected gpio wait satisfied, got {other:?}"),
        };
        assert_eq!(
            edge,
            GpioEdge::new(wait.wait_id(), wait.pin(), true, wait.generation())
        );

        (kernel
            .flow::<GpioWaitRetMsg>()
            .expect("kernel flow<gpio wait ret>")
            .send(&edge))
        .await
        .expect("kernel sends gpio wait ret");

        let received_edge = (engine.recv::<GpioWaitRetMsg>())
            .await
            .expect("engine receives gpio wait ret");
        assert_eq!(received_edge, edge);
        assert_eq!(resolver.resolve_next(), Ok(None));
    });
}
