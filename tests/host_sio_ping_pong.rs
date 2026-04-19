use hibana::{
    g,
    g::advanced::steps::{SendStep, SeqSteps, StepCons, StepNil},
    g::advanced::{RoleProgram, project},
    g::{Msg, Role},
    substrate::{
        SessionId, SessionKit,
        binding::NoBinding,
        runtime::{Config, CounterClock, LabelUniverse},
        tap::TapEvent,
    },
};
use hibana_pico::{backend::HostQueueBackend, exec::drive, transport::SioTransport};

const LABEL_PING: u8 = 1;
const LABEL_PONG: u8 = 2;
const PING_VALUE: u8 = 0x2a;
const PONG_VALUE: u8 = 0x55;

#[derive(Clone, Copy, Debug, Default)]
struct PingPongLabelUniverse;

impl LabelUniverse for PingPongLabelUniverse {
    const MAX_LABEL: u8 = LABEL_PONG;
}

type PingStep = StepCons<SendStep<Role<1>, Role<0>, Msg<LABEL_PING, u8>, 0>, StepNil>;
type PongStep = StepCons<SendStep<Role<0>, Role<1>, Msg<LABEL_PONG, u8>, 0>, StepNil>;
type ProgramSteps = SeqSteps<PingStep, PongStep>;

const PROGRAM: g::Program<ProgramSteps> = g::seq(
    g::send::<Role<1>, Role<0>, Msg<LABEL_PING, u8>, 0>(),
    g::send::<Role<0>, Role<1>, Msg<LABEL_PONG, u8>, 0>(),
);

static CORE0_PROGRAM: RoleProgram<'static, 0> = project(&PROGRAM);
static CORE1_PROGRAM: RoleProgram<'static, 1> = project(&PROGRAM);

type TestTransport<'a> = SioTransport<&'a HostQueueBackend>;
type TestKit<'a> = SessionKit<'a, TestTransport<'a>, PingPongLabelUniverse, CounterClock, 1>;

#[test]
fn host_backend_roundtrips_hibana_localside_ping_pong() {
    let backend = HostQueueBackend::new();

    let clock0 = CounterClock::new();
    let mut tap0 = [TapEvent::zero(); 128];
    let mut slab0 = [0u8; 262_144];
    let cluster0 = TestKit::new(&clock0);
    let rv0 = cluster0
        .add_rendezvous_from_config(
            Config::new(&mut tap0, &mut slab0).with_universe(PingPongLabelUniverse),
            SioTransport::new(&backend),
        )
        .expect("register core0 rendezvous");

    let clock1 = CounterClock::new();
    let mut tap1 = [TapEvent::zero(); 128];
    let mut slab1 = [0u8; 262_144];
    let cluster1 = TestKit::new(&clock1);
    let rv1 = cluster1
        .add_rendezvous_from_config(
            Config::new(&mut tap1, &mut slab1).with_universe(PingPongLabelUniverse),
            SioTransport::new(&backend),
        )
        .expect("register core1 rendezvous");

    let sid = SessionId::new(17);
    let mut core0 = cluster0
        .enter(rv0, sid, &CORE0_PROGRAM, NoBinding)
        .expect("attach core0 endpoint");
    let mut core1 = cluster1
        .enter(rv1, sid, &CORE1_PROGRAM, NoBinding)
        .expect("attach core1 endpoint");

    let _ = drive(
        core1
            .flow::<Msg<LABEL_PING, u8>>()
            .expect("core1 flow<ping>")
            .send(&PING_VALUE),
    )
    .expect("core1 send ping");

    let ping = drive(core0.recv::<Msg<LABEL_PING, u8>>()).expect("core0 recv ping");
    assert_eq!(ping, PING_VALUE);

    let _ = drive(
        core0
            .flow::<Msg<LABEL_PONG, u8>>()
            .expect("core0 flow<pong>")
            .send(&PONG_VALUE),
    )
    .expect("core0 send pong");

    let pong = drive(core1.recv::<Msg<LABEL_PONG, u8>>()).expect("core1 recv pong");
    assert_eq!(pong, PONG_VALUE);
}
