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
        EngineLabelUniverse, EngineReq, EngineRet, LABEL_PUBLISH_ALERT, LABEL_PUBLISH_NORMAL,
        LABEL_SAMPLE_REQ, LABEL_YIELD_REQ, LABEL_YIELD_RET, PublishAlertControl,
        PublishNormalControl,
    },
    kernel::engine::wasm::{
        BAD_ROUTE_EARLY_YIELD_WASM_GUEST, DEMO_WASM_GUEST, GuestTrap, NORMAL_WASM_GUEST,
        ROUTE_WASM_ALERT_VALUE, ROUTE_WASM_NORMAL_VALUE, TinyWasmInstance,
    },
    substrate::host_queue::HostQueueBackend,
    substrate::transport::SioTransport,
};

type TestTransport<'a> = SioTransport<&'a HostQueueBackend>;
type TestKit<'a> = SessionKit<'a, TestTransport<'a>, EngineLabelUniverse, CounterClock, 2>;

fn project_route_roles() -> (RoleProgram<0>, RoleProgram<1>) {
    let publish_normal_arm = g::seq(
        g::send::<Role<0>, Role<0>, PublishNormalControl, 0>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_PUBLISH_NORMAL, u32>, 0>(),
    );
    let publish_alert_arm = g::seq(
        g::send::<Role<0>, Role<0>, PublishAlertControl, 0>(),
        g::send::<Role<0>, Role<1>, Msg<LABEL_PUBLISH_ALERT, u32>, 0>(),
    );
    let program = g::seq(
        g::send::<Role<1>, Role<0>, Msg<LABEL_SAMPLE_REQ, u32>, 0>(),
        g::seq(
            g::route(publish_normal_arm, publish_alert_arm),
            g::seq(
                g::send::<Role<1>, Role<0>, Msg<LABEL_YIELD_REQ, ()>, 0>(),
                g::send::<Role<0>, Role<1>, Msg<LABEL_YIELD_RET, ()>, 0>(),
            ),
        ),
    );
    let core0: RoleProgram<0> = project(&program);
    let core1: RoleProgram<1> = project(&program);
    (core0, core1)
}

async fn run_route_guest(guest_bytes: &[u8], sample_value: u32, expected_branch_label: u8) {
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

    let sid = SessionId::new(41);
    let (core0_program, core1_program) = project_route_roles();
    let mut supervisor = cluster0
        .enter(rv0, sid, &core0_program, NoBinding)
        .expect("attach supervisor endpoint");
    let mut engine = cluster1
        .enter(rv1, sid, &core1_program, NoBinding)
        .expect("attach engine endpoint");

    let mut guest = TinyWasmInstance::new(guest_bytes).expect("instantiate guest");

    let trap = guest.resume().expect("resume to sample");
    assert_eq!(trap, GuestTrap::HostCall(EngineReq::LogU32(sample_value)));
    let _ = (engine
        .flow::<Msg<LABEL_SAMPLE_REQ, u32>>()
        .expect("engine flow<sample req>")
        .send(&sample_value))
    .await
    .expect("engine send sample");
    let sample = (supervisor.recv::<Msg<LABEL_SAMPLE_REQ, u32>>())
        .await
        .expect("supervisor recv sample");
    assert_eq!(sample, sample_value);
    assert!(
        supervisor.flow::<Msg<LABEL_YIELD_RET, ()>>().is_err(),
        "yield ack must not be available before publish route is closed"
    );

    match expected_branch_label {
        LABEL_PUBLISH_NORMAL => {
            (supervisor
                .flow::<PublishNormalControl>()
                .expect("supervisor flow<publish normal control>")
                .send(()))
            .await
            .expect("supervisor route normal");
            let _ = (supervisor
                .flow::<Msg<LABEL_PUBLISH_NORMAL, u32>>()
                .expect("supervisor flow<publish normal>")
                .send(&sample))
            .await
            .expect("supervisor send publish normal");
        }
        LABEL_PUBLISH_ALERT => {
            (supervisor
                .flow::<PublishAlertControl>()
                .expect("supervisor flow<publish alert control>")
                .send(()))
            .await
            .expect("supervisor route alert");
            let _ = (supervisor
                .flow::<Msg<LABEL_PUBLISH_ALERT, u32>>()
                .expect("supervisor flow<publish alert>")
                .send(&sample))
            .await
            .expect("supervisor send publish alert");
        }
        _ => panic!("unexpected branch label"),
    }

    let branch = (engine.offer()).await.expect("engine offer publish");
    assert_eq!(branch.label(), expected_branch_label);
    let published = match expected_branch_label {
        LABEL_PUBLISH_NORMAL => (branch.decode::<Msg<LABEL_PUBLISH_NORMAL, u32>>())
            .await
            .expect("engine decode publish normal"),
        LABEL_PUBLISH_ALERT => (branch.decode::<Msg<LABEL_PUBLISH_ALERT, u32>>())
            .await
            .expect("engine decode publish alert"),
        _ => unreachable!(),
    };
    assert_eq!(published, sample_value);
    guest
        .complete_host_call(EngineRet::Logged(published))
        .expect("complete sample");

    assert_eq!(
        guest.resume().expect("resume to yield"),
        GuestTrap::HostCall(EngineReq::Yield)
    );
    let _ = (engine
        .flow::<Msg<LABEL_YIELD_REQ, ()>>()
        .expect("engine flow<yield req>")
        .send(&()))
    .await
    .expect("engine send yield");
    (supervisor.recv::<Msg<LABEL_YIELD_REQ, ()>>())
        .await
        .expect("supervisor recv yield");
    let _ = (supervisor
        .flow::<Msg<LABEL_YIELD_RET, ()>>()
        .expect("supervisor flow<yield ret>")
        .send(&()))
    .await
    .expect("supervisor send yield ack");
    (engine.recv::<Msg<LABEL_YIELD_RET, ()>>())
        .await
        .expect("engine recv yield ack");
    guest
        .complete_host_call(EngineRet::Yielded)
        .expect("complete yield");

    assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
}

#[test]
fn host_backend_roundtrips_normal_publish_branch() {
    hibana_pico::substrate::exec::run_current_task(async {
        run_route_guest(
            NORMAL_WASM_GUEST,
            ROUTE_WASM_NORMAL_VALUE,
            LABEL_PUBLISH_NORMAL,
        )
        .await;
    });
}

#[test]
fn host_backend_roundtrips_alert_publish_branch() {
    hibana_pico::substrate::exec::run_current_task(async {
        run_route_guest(DEMO_WASM_GUEST, ROUTE_WASM_ALERT_VALUE, LABEL_PUBLISH_ALERT).await;
    });
}

#[test]
fn host_backend_rejects_early_yield_before_sample_route() {
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

        let sid = SessionId::new(42);
        let (core0_program, core1_program) = project_route_roles();
        let _supervisor = cluster0
            .enter(rv0, sid, &core0_program, NoBinding)
            .expect("attach supervisor endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &core1_program, NoBinding)
            .expect("attach engine endpoint");

        let mut guest =
            TinyWasmInstance::new(BAD_ROUTE_EARLY_YIELD_WASM_GUEST).expect("instantiate bad guest");

        assert!(
            engine.flow::<Msg<LABEL_SAMPLE_REQ, u32>>().is_ok(),
            "sample req must be the first reachable localside action"
        );
        assert_eq!(
            guest.resume().expect("resume to early yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
        assert!(
            engine.flow::<Msg<LABEL_YIELD_REQ, ()>>().is_err(),
            "yield req must be rejected until sample and publish route finish"
        );
    });
}
