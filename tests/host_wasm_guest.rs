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
        EngineLabelUniverse, EngineReq, EngineRet, LABEL_ENGINE_REQ, LABEL_ENGINE_RET,
    },
    kernel::engine::wasm::{DEMO_WASM_GUEST, GuestTrap, TinyWasmInstance},
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

#[test]
fn host_backend_roundtrips_wasm_guest_requests() {
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

        let sid = SessionId::new(29);
        let (core0_program, core1_program) = project_engine_roles();
        let mut supervisor = cluster0
            .enter(rv0, sid, &core0_program, NoBinding)
            .expect("attach supervisor endpoint");
        let mut engine = cluster1
            .enter(rv1, sid, &core1_program, NoBinding)
            .expect("attach engine endpoint");

        let mut guest = TinyWasmInstance::new(DEMO_WASM_GUEST).expect("instantiate guest");

        let trap = guest.resume().expect("resume to log_u32");
        assert_eq!(
            trap,
            GuestTrap::HostCall(EngineReq::LogU32(ENGINE_LOG_VALUE))
        );
        if let GuestTrap::HostCall(request) = trap {
            let _ = (engine
                .flow::<Msg<LABEL_ENGINE_REQ, EngineReq>>()
                .expect("engine flow<req>")
                .send(&request))
            .await
            .expect("engine send log_u32");
        }
        let received = (supervisor.recv::<Msg<LABEL_ENGINE_REQ, EngineReq>>())
            .await
            .expect("supervisor recv log_u32");
        assert_eq!(received, EngineReq::LogU32(ENGINE_LOG_VALUE));
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
        guest
            .complete_host_call(received)
            .expect("complete log call");

        let trap = guest.resume().expect("resume to yield");
        assert_eq!(trap, GuestTrap::HostCall(EngineReq::Yield));
        if let GuestTrap::HostCall(request) = trap {
            let _ = (engine
                .flow::<Msg<LABEL_ENGINE_REQ, EngineReq>>()
                .expect("engine flow<req>")
                .send(&request))
            .await
            .expect("engine send yield");
        }
        let received = (supervisor.recv::<Msg<LABEL_ENGINE_REQ, EngineReq>>())
            .await
            .expect("supervisor recv yield");
        assert_eq!(received, EngineReq::Yield);
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
        guest
            .complete_host_call(received)
            .expect("complete yield call");

        assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
    });
}
