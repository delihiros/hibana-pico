use hibana::{
    g::Msg,
    substrate::{
        cap::{
            CapShot, ControlResourceKind, GenericCapToken, ResourceKind,
            advanced::{
                CAP_HANDLE_LEN, CapError, ControlOp, ControlPath, ControlScopeKind, LoopBreakKind,
                LoopContinueKind, RouteDecisionKind, ScopeId,
            },
        },
        ids::{Lane, SessionId},
        runtime::LabelUniverse,
        wire::{CodecError, Payload, WireEncode, WirePayload},
    },
};

pub const LABEL_ENGINE_REQ: u8 = 1;
pub const LABEL_ENGINE_RET: u8 = 2;
pub const LABEL_ROUTE_PUBLISH_NORMAL: u8 = 118;
pub const LABEL_ROUTE_PUBLISH_ALERT: u8 = 119;
pub const LABEL_SAMPLE_REQ: u8 = 12;
pub const LABEL_PUBLISH_NORMAL: u8 = 13;
pub const LABEL_PUBLISH_ALERT: u8 = 14;
pub const LABEL_YIELD_REQ: u8 = 15;
pub const LABEL_YIELD_RET: u8 = 16;
pub const LABEL_WASIP1_STDOUT: u8 = 17;
pub const LABEL_WASIP1_STDOUT_RET: u8 = 18;
pub const LABEL_WASIP1_STDERR: u8 = 19;
pub const LABEL_WASIP1_STDERR_RET: u8 = 20;
pub const LABEL_WASIP1_STDIN: u8 = 21;
pub const LABEL_WASIP1_STDIN_RET: u8 = 22;
pub const LABEL_WASIP1_CLOCK_NOW: u8 = 23;
pub const LABEL_WASIP1_CLOCK_NOW_RET: u8 = 24;
pub const LABEL_WASIP1_RANDOM_SEED: u8 = 25;
pub const LABEL_WASIP1_RANDOM_SEED_RET: u8 = 26;
pub const LABEL_WASIP1_EXIT: u8 = 27;
pub const LABEL_MEM_BORROW_READ: u8 = 28;
pub const LABEL_MEM_BORROW_WRITE: u8 = 29;
pub const LABEL_MEM_GRANT: u8 = 30;
pub const LABEL_MEM_RELEASE: u8 = 31;
pub const LABEL_MEM_COMMIT: u8 = 32;
pub const LABEL_MEM_FENCE: u8 = 33;
pub const LABEL_TIMER_SLEEP_UNTIL: u8 = 34;
pub const LABEL_TIMER_SLEEP_DONE: u8 = 35;
pub const LABEL_GPIO_SET: u8 = 36;
pub const LABEL_GPIO_SET_DONE: u8 = 37;
pub const LABEL_UART_WRITE: u8 = 38;
pub const LABEL_UART_WRITE_RET: u8 = 39;
pub const LABEL_MGMT_IMAGE_BEGIN: u8 = 40;
pub const LABEL_MGMT_IMAGE_CHUNK: u8 = 41;
pub const LABEL_MGMT_IMAGE_END: u8 = 42;
pub const LABEL_MGMT_IMAGE_STATUS: u8 = 43;
pub const LABEL_MGMT_IMAGE_ACTIVATE: u8 = 44;
pub const LABEL_MGMT_IMAGE_ROLLBACK: u8 = 45;
pub const LABEL_ENGINE_RUN: u8 = 50;
pub const LABEL_ENGINE_BUDGET_EXPIRED: u8 = 51;
pub const LABEL_ENGINE_SUSPEND: u8 = 52;
pub const LABEL_ENGINE_RESTART: u8 = 53;
pub const LABEL_GPIO_WAIT: u8 = 54;
pub const LABEL_GPIO_SUBSCRIBE: u8 = 55;
pub const LABEL_GPIO_EDGE: u8 = 56;
pub const LABEL_GPIO_WAIT_RET: u8 = 58;
pub const LABEL_NET_STREAM_WRITE: u8 = 59;
pub const LABEL_NET_STREAM_ACK: u8 = 60;
pub const LABEL_NET_STREAM_READ: u8 = 61;
pub const LABEL_NET_STREAM_READ_RET: u8 = 62;
pub const LABEL_WASI_CLOCK_RES_GET: u8 = 63;
pub const LABEL_WASI_CLOCK_RES_GET_RET: u8 = 64;
pub const LABEL_WASI_FD_WRITE: u8 = 85;
pub const LABEL_WASI_FD_WRITE_RET: u8 = 86;
pub const LABEL_WASI_FD_READ: u8 = 87;
pub const LABEL_WASI_FD_READ_RET: u8 = 88;
pub const LABEL_WASI_FD_FDSTAT_GET: u8 = 89;
pub const LABEL_WASI_FD_FDSTAT_GET_RET: u8 = 90;
pub const LABEL_WASI_FD_CLOSE: u8 = 91;
pub const LABEL_WASI_FD_CLOSE_RET: u8 = 92;
pub const LABEL_WASI_CLOCK_TIME_GET: u8 = 93;
pub const LABEL_WASI_CLOCK_TIME_GET_RET: u8 = 94;
pub const LABEL_WASI_POLL_ONEOFF: u8 = 95;
pub const LABEL_WASI_POLL_ONEOFF_RET: u8 = 96;
pub const LABEL_WASI_RANDOM_GET: u8 = 97;
pub const LABEL_WASI_RANDOM_GET_RET: u8 = 98;
pub const LABEL_WASI_PROC_EXIT: u8 = 99;
pub const LABEL_WASI_ARGS_GET: u8 = 100;
pub const LABEL_WASI_ARGS_GET_RET: u8 = 101;
pub const LABEL_WASI_ENVIRON_GET: u8 = 102;
pub const LABEL_WASI_ENVIRON_GET_RET: u8 = 103;
pub const LABEL_WASI_FD_ERROR: u8 = 104;
pub const LABEL_REMOTE_SAMPLE_REQ: u8 = 65;
pub const LABEL_REMOTE_SAMPLE_RET: u8 = 66;
pub const LABEL_REMOTE_ACTUATE_REQ: u8 = 67;
pub const LABEL_REMOTE_ACTUATE_RET: u8 = 68;
pub const LABEL_NET_DATAGRAM_SEND: u8 = 69;
pub const LABEL_NET_DATAGRAM_ACK: u8 = 70;
pub const LABEL_NET_DATAGRAM_RECV: u8 = 71;
pub const LABEL_NET_DATAGRAM_RECV_RET: u8 = 72;
pub const LABEL_SWARM_TELEMETRY: u8 = 73;
pub const LABEL_SWARM_NODE_IMAGE_UPDATED: u8 = 74;
pub const LABEL_SWARM_JOIN_OFFER: u8 = 75;
pub const LABEL_SWARM_JOIN_GRANT: u8 = 76;
pub const LABEL_SWARM_NODE_REVOKED: u8 = 77;
pub const LABEL_SWARM_POLICY_APP0: u8 = 78;
pub const LABEL_SWARM_POLICY_APP1: u8 = 79;
pub const LABEL_SWARM_JOIN_REQUEST: u8 = 80;
pub const LABEL_SWARM_JOIN_ACK: u8 = 81;
pub const LABEL_SWARM_SUSPEND: u8 = 82;
pub const LABEL_SWARM_REVOKE_REMOTE_OBJECTS: u8 = 83;
pub const LABEL_SWARM_LEAVE_ACK: u8 = 84;
pub const LABEL_MEM_GRANT_READ_CONTROL: u8 = 106;
pub const LABEL_MEM_GRANT_WRITE_CONTROL: u8 = 107;
pub const LABEL_ROUTE_REMOTE_SENSOR: u8 = 108;
pub const LABEL_ROUTE_REMOTE_ACTUATOR: u8 = 109;
pub const LABEL_ROUTE_REMOTE_REJECT: u8 = 110;
pub const LABEL_ROUTE_NETWORK_DATAGRAM_SEND: u8 = 111;
pub const LABEL_ROUTE_NETWORK_DATAGRAM_RECV: u8 = 112;
pub const LABEL_ROUTE_NETWORK_REJECT: u8 = 113;
pub const LABEL_ROUTE_NETWORK_STREAM_WRITE: u8 = 114;
pub const LABEL_ROUTE_NETWORK_STREAM_READ: u8 = 115;
pub const LABEL_ROUTE_NETWORK_ACCEPT: u8 = 122;
pub const LABEL_ROUTE_REMOTE_MANAGEMENT: u8 = 116;
pub const LABEL_ROUTE_REMOTE_TELEMETRY: u8 = 117;
pub const LABEL_BAKER_TRAFFIC_LOOP_CONTINUE: u8 = 120;
pub const LABEL_BAKER_TRAFFIC_LOOP_BREAK: u8 = 121;
pub const POLICY_BAKER_TRAFFIC_LOOP: u16 = 120;

const TAG_REQ_LOG_U32: u8 = 1;
const TAG_REQ_YIELD: u8 = 2;
const TAG_REQ_WASIP1_STDOUT: u8 = 3;
const TAG_REQ_WASIP1_STDERR: u8 = 4;
const TAG_REQ_WASIP1_STDIN: u8 = 5;
const TAG_REQ_WASIP1_CLOCK_NOW: u8 = 6;
const TAG_REQ_WASIP1_RANDOM_SEED: u8 = 7;
const TAG_REQ_WASIP1_EXIT: u8 = 8;
const TAG_REQ_TIMER_SLEEP_UNTIL: u8 = 9;
const TAG_REQ_GPIO_SET: u8 = 10;
const TAG_REQ_WASI_FD_WRITE: u8 = 11;
const TAG_REQ_WASI_FD_READ: u8 = 12;
const TAG_REQ_WASI_FD_FDSTAT_GET: u8 = 13;
const TAG_REQ_WASI_FD_CLOSE: u8 = 14;
const TAG_REQ_WASI_CLOCK_TIME_GET: u8 = 15;
const TAG_REQ_WASI_POLL_ONEOFF: u8 = 16;
const TAG_REQ_WASI_RANDOM_GET: u8 = 17;
const TAG_REQ_WASI_PROC_EXIT: u8 = 18;
const TAG_REQ_WASI_ARGS_GET: u8 = 19;
const TAG_REQ_WASI_ENVIRON_GET: u8 = 20;
const TAG_REQ_WASI_CLOCK_RES_GET: u8 = 21;
const TAG_RET_LOGGED: u8 = 1;
const TAG_RET_YIELDED: u8 = 2;
const TAG_RET_WASIP1_STDOUT_WRITTEN: u8 = 3;
const TAG_RET_WASIP1_STDERR_WRITTEN: u8 = 4;
const TAG_RET_WASIP1_STDIN_READ: u8 = 5;
const TAG_RET_WASIP1_CLOCK_NOW: u8 = 6;
const TAG_RET_WASIP1_RANDOM_SEED: u8 = 7;
const TAG_RET_TIMER_SLEEP_DONE: u8 = 8;
const TAG_RET_GPIO_SET_DONE: u8 = 9;
const TAG_RET_WASI_FD_WRITE_DONE: u8 = 10;
const TAG_RET_WASI_FD_READ_DONE: u8 = 11;
const TAG_RET_WASI_FD_FDSTAT: u8 = 12;
const TAG_RET_WASI_FD_CLOSED: u8 = 13;
const TAG_RET_WASI_CLOCK_TIME: u8 = 14;
const TAG_RET_WASI_POLL_READY: u8 = 15;
const TAG_RET_WASI_RANDOM_DONE: u8 = 16;
const TAG_RET_WASI_ARGS_DONE: u8 = 17;
const TAG_RET_WASI_ENVIRON_DONE: u8 = 18;
const TAG_RET_WASI_CLOCK_RESOLUTION: u8 = 19;

pub const WASIP1_STREAM_CHUNK_CAPACITY: usize = 30;
pub const STDOUT_CHUNK_CAPACITY: usize = WASIP1_STREAM_CHUNK_CAPACITY;
pub const STDERR_CHUNK_CAPACITY: usize = WASIP1_STREAM_CHUNK_CAPACITY;
pub const STDIN_CHUNK_CAPACITY: usize = WASIP1_STREAM_CHUNK_CAPACITY;
pub const UART_WRITE_CHUNK_CAPACITY: usize = 64;
pub const MGMT_IMAGE_CHUNK_CAPACITY: usize = 32;
pub const MEM_LEASE_NONE: u8 = 0;

type RouteWireHandle = (u8, u64);
pub type MemoryLeaseWireHandle = (u8, u64);

#[derive(Clone, Copy, Debug, Default)]
pub struct EngineLabelUniverse;

impl LabelUniverse for EngineLabelUniverse {
    const MAX_LABEL: u8 = LABEL_ROUTE_NETWORK_ACCEPT;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteControl<const KIND_LABEL: u8, const ARM: u8>;

impl<const KIND_LABEL: u8, const ARM: u8> ResourceKind for RouteControl<KIND_LABEL, ARM> {
    type Handle = RouteWireHandle;
    const TAG: u8 = <RouteDecisionKind as ResourceKind>::TAG;
    const NAME: &'static str = "RouteControl";

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        let mut buf = [0u8; CAP_HANDLE_LEN];
        buf[0] = handle.0;
        buf[1..9].copy_from_slice(&handle.1.to_le_bytes());
        buf
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        let mut scope_bytes = [0u8; 8];
        scope_bytes.copy_from_slice(&data[1..9]);
        Ok((data[0], u64::from_le_bytes(scope_bytes)))
    }

    fn zeroize(handle: &mut Self::Handle) {
        *handle = (0, 0);
    }
}

impl<const KIND_LABEL: u8, const ARM: u8> ControlResourceKind for RouteControl<KIND_LABEL, ARM> {
    const SCOPE: ControlScopeKind = ControlScopeKind::Route;
    const TAP_ID: u16 = <RouteDecisionKind as ControlResourceKind>::TAP_ID;
    const SHOT: CapShot = CapShot::One;
    const PATH: ControlPath = ControlPath::Local;
    const OP: ControlOp = ControlOp::RouteDecision;
    const AUTO_MINT_WIRE: bool = false;

    fn mint_handle(_sid: SessionId, _lane: Lane, scope: ScopeId) -> <Self as ResourceKind>::Handle {
        (ARM, scope.raw())
    }
}

pub type PublishNormalKind = RouteControl<LABEL_ROUTE_PUBLISH_NORMAL, 0>;
pub type PublishAlertKind = RouteControl<LABEL_ROUTE_PUBLISH_ALERT, 1>;
pub type PublishNormalControl =
    Msg<LABEL_ROUTE_PUBLISH_NORMAL, GenericCapToken<PublishNormalKind>, PublishNormalKind>;
pub type PublishAlertControl =
    Msg<LABEL_ROUTE_PUBLISH_ALERT, GenericCapToken<PublishAlertKind>, PublishAlertKind>;
pub type RemoteSensorRouteKind = RouteControl<LABEL_ROUTE_REMOTE_SENSOR, 0>;
pub type RemoteActuatorRouteKind = RouteControl<LABEL_ROUTE_REMOTE_ACTUATOR, 1>;
pub type RemoteManagementRouteKind = RouteControl<LABEL_ROUTE_REMOTE_MANAGEMENT, 2>;
pub type RemoteTelemetryRouteKind = RouteControl<LABEL_ROUTE_REMOTE_TELEMETRY, 3>;
pub type RemoteRejectRouteKind = RouteControl<LABEL_ROUTE_REMOTE_REJECT, 4>;
pub type RemoteSensorRouteControl =
    Msg<LABEL_ROUTE_REMOTE_SENSOR, GenericCapToken<RemoteSensorRouteKind>, RemoteSensorRouteKind>;
pub type RemoteActuatorRouteControl = Msg<
    LABEL_ROUTE_REMOTE_ACTUATOR,
    GenericCapToken<RemoteActuatorRouteKind>,
    RemoteActuatorRouteKind,
>;
pub type RemoteManagementRouteControl = Msg<
    LABEL_ROUTE_REMOTE_MANAGEMENT,
    GenericCapToken<RemoteManagementRouteKind>,
    RemoteManagementRouteKind,
>;
pub type RemoteTelemetryRouteControl = Msg<
    LABEL_ROUTE_REMOTE_TELEMETRY,
    GenericCapToken<RemoteTelemetryRouteKind>,
    RemoteTelemetryRouteKind,
>;
pub type RemoteRejectRouteControl =
    Msg<LABEL_ROUTE_REMOTE_REJECT, GenericCapToken<RemoteRejectRouteKind>, RemoteRejectRouteKind>;
pub type NetworkDatagramSendRouteKind = RouteControl<LABEL_ROUTE_NETWORK_DATAGRAM_SEND, 0>;
pub type NetworkDatagramRecvRouteKind = RouteControl<LABEL_ROUTE_NETWORK_DATAGRAM_RECV, 1>;
pub type NetworkStreamWriteRouteKind = RouteControl<LABEL_ROUTE_NETWORK_STREAM_WRITE, 2>;
pub type NetworkStreamReadRouteKind = RouteControl<LABEL_ROUTE_NETWORK_STREAM_READ, 3>;
pub type NetworkRejectRouteKind = RouteControl<LABEL_ROUTE_NETWORK_REJECT, 4>;
pub type NetworkAcceptRouteKind = RouteControl<LABEL_ROUTE_NETWORK_ACCEPT, 5>;
pub type NetworkDatagramSendRouteControl = Msg<
    LABEL_ROUTE_NETWORK_DATAGRAM_SEND,
    GenericCapToken<NetworkDatagramSendRouteKind>,
    NetworkDatagramSendRouteKind,
>;
pub type NetworkDatagramRecvRouteControl = Msg<
    LABEL_ROUTE_NETWORK_DATAGRAM_RECV,
    GenericCapToken<NetworkDatagramRecvRouteKind>,
    NetworkDatagramRecvRouteKind,
>;
pub type NetworkStreamWriteRouteControl = Msg<
    LABEL_ROUTE_NETWORK_STREAM_WRITE,
    GenericCapToken<NetworkStreamWriteRouteKind>,
    NetworkStreamWriteRouteKind,
>;
pub type NetworkStreamReadRouteControl = Msg<
    LABEL_ROUTE_NETWORK_STREAM_READ,
    GenericCapToken<NetworkStreamReadRouteKind>,
    NetworkStreamReadRouteKind,
>;
pub type NetworkRejectRouteControl = Msg<
    LABEL_ROUTE_NETWORK_REJECT,
    GenericCapToken<NetworkRejectRouteKind>,
    NetworkRejectRouteKind,
>;
pub type NetworkAcceptRouteControl = Msg<
    LABEL_ROUTE_NETWORK_ACCEPT,
    GenericCapToken<NetworkAcceptRouteKind>,
    NetworkAcceptRouteKind,
>;
pub type BakerTrafficLoopContinueControl =
    Msg<LABEL_BAKER_TRAFFIC_LOOP_CONTINUE, GenericCapToken<LoopContinueKind>, LoopContinueKind>;
pub type BakerTrafficLoopBreakControl =
    Msg<LABEL_BAKER_TRAFFIC_LOOP_BREAK, GenericCapToken<LoopBreakKind>, LoopBreakKind>;
pub type BudgetRunMsg = Msg<LABEL_ENGINE_RUN, BudgetRun>;
pub type BudgetExpiredMsg = Msg<LABEL_ENGINE_BUDGET_EXPIRED, BudgetExpired>;
pub type BudgetSuspendMsg = Msg<LABEL_ENGINE_SUSPEND, BudgetSuspend>;
pub type BudgetRestartMsg = Msg<LABEL_ENGINE_RESTART, BudgetRestart>;
pub type GpioWaitMsg = Msg<LABEL_GPIO_WAIT, GpioWait>;
pub type GpioSubscribeMsg = Msg<LABEL_GPIO_SUBSCRIBE, GpioWait>;
pub type GpioEdgeMsg = Msg<LABEL_GPIO_EDGE, GpioEdge>;
pub type GpioWaitRetMsg = Msg<LABEL_GPIO_WAIT_RET, GpioEdge>;
pub type UartWriteMsg = Msg<LABEL_UART_WRITE, UartWrite>;
pub type UartWriteRetMsg = Msg<LABEL_UART_WRITE_RET, UartWriteDone>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemReadLeaseKind;

impl ResourceKind for MemReadLeaseKind {
    type Handle = MemoryLeaseWireHandle;
    const TAG: u8 = 0x31;
    const NAME: &'static str = "MemReadLease";

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        encode_memory_lease_handle(*handle)
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        decode_memory_lease_handle(data)
    }

    fn zeroize(handle: &mut Self::Handle) {
        *handle = (0, 0);
    }
}

impl ControlResourceKind for MemReadLeaseKind {
    const SCOPE: ControlScopeKind = ControlScopeKind::Policy;
    const TAP_ID: u16 = 0x04d0;
    const SHOT: CapShot = CapShot::One;
    const PATH: ControlPath = ControlPath::Wire;
    const OP: ControlOp = ControlOp::Fence;
    const AUTO_MINT_WIRE: bool = true;

    fn mint_handle(
        _sid: SessionId,
        _lane: Lane,
        _scope: ScopeId,
    ) -> <Self as ResourceKind>::Handle {
        (MemRights::Read.tag(), 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemWriteLeaseKind;

impl ResourceKind for MemWriteLeaseKind {
    type Handle = MemoryLeaseWireHandle;
    const TAG: u8 = 0x32;
    const NAME: &'static str = "MemWriteLease";

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        encode_memory_lease_handle(*handle)
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        decode_memory_lease_handle(data)
    }

    fn zeroize(handle: &mut Self::Handle) {
        *handle = (0, 0);
    }
}

impl ControlResourceKind for MemWriteLeaseKind {
    const SCOPE: ControlScopeKind = ControlScopeKind::Policy;
    const TAP_ID: u16 = 0x04d1;
    const SHOT: CapShot = CapShot::One;
    const PATH: ControlPath = ControlPath::Wire;
    const OP: ControlOp = ControlOp::Fence;
    const AUTO_MINT_WIRE: bool = true;

    fn mint_handle(
        _sid: SessionId,
        _lane: Lane,
        _scope: ScopeId,
    ) -> <Self as ResourceKind>::Handle {
        (MemRights::Write.tag(), 1)
    }
}

pub type MemReadGrantControl =
    Msg<LABEL_MEM_GRANT_READ_CONTROL, GenericCapToken<MemReadLeaseKind>, MemReadLeaseKind>;
pub type MemWriteGrantControl =
    Msg<LABEL_MEM_GRANT_WRITE_CONTROL, GenericCapToken<MemWriteLeaseKind>, MemWriteLeaseKind>;

fn encode_memory_lease_handle(handle: MemoryLeaseWireHandle) -> [u8; CAP_HANDLE_LEN] {
    let mut buf = [0u8; CAP_HANDLE_LEN];
    buf[0] = handle.0;
    buf[1..9].copy_from_slice(&handle.1.to_le_bytes());
    buf
}

fn decode_memory_lease_handle(
    data: [u8; CAP_HANDLE_LEN],
) -> Result<MemoryLeaseWireHandle, CapError> {
    let mut lease_bytes = [0u8; 8];
    lease_bytes.copy_from_slice(&data[1..9]);
    Ok((data[0], u64::from_le_bytes(lease_bytes)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemRights {
    Read,
    Write,
}

impl MemRights {
    pub const fn tag(self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Write => 2,
        }
    }

    fn decode(tag: u8) -> Result<Self, CodecError> {
        match tag {
            1 => Ok(Self::Read),
            2 => Ok(Self::Write),
            _ => Err(CodecError::Invalid("unknown memory lease rights")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemBorrow {
    ptr: u32,
    len: u8,
    epoch: u32,
}

impl MemBorrow {
    pub const fn new(ptr: u32, len: u8, epoch: u32) -> Self {
        Self { ptr, len, epoch }
    }

    pub const fn ptr(&self) -> u32 {
        self.ptr
    }

    pub const fn len(&self) -> u8 {
        self.len
    }

    pub const fn epoch(&self) -> u32 {
        self.epoch
    }
}

impl WireEncode for MemBorrow {
    fn encoded_len(&self) -> Option<usize> {
        Some(9)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 9 {
            return Err(CodecError::Truncated);
        }
        out[..4].copy_from_slice(&self.ptr.to_be_bytes());
        out[4] = self.len;
        out[5..9].copy_from_slice(&self.epoch.to_be_bytes());
        Ok(9)
    }
}

impl WirePayload for MemBorrow {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 9 {
            return Err(CodecError::Invalid("memory borrow carries nine bytes"));
        }
        let mut ptr = [0u8; 4];
        let mut epoch = [0u8; 4];
        ptr.copy_from_slice(&bytes[..4]);
        epoch.copy_from_slice(&bytes[5..9]);
        Ok(Self::new(
            u32::from_be_bytes(ptr),
            bytes[4],
            u32::from_be_bytes(epoch),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemGrant {
    lease_id: u8,
    ptr: u32,
    len: u8,
    epoch: u32,
    rights: MemRights,
}

impl MemGrant {
    pub const fn new(lease_id: u8, ptr: u32, len: u8, epoch: u32, rights: MemRights) -> Self {
        Self {
            lease_id,
            ptr,
            len,
            epoch,
            rights,
        }
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn ptr(&self) -> u32 {
        self.ptr
    }

    pub const fn len(&self) -> u8 {
        self.len
    }

    pub const fn epoch(&self) -> u32 {
        self.epoch
    }

    pub const fn rights(&self) -> MemRights {
        self.rights
    }
}

impl WireEncode for MemGrant {
    fn encoded_len(&self) -> Option<usize> {
        Some(11)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 11 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.lease_id;
        out[1..5].copy_from_slice(&self.ptr.to_be_bytes());
        out[5] = self.len;
        out[6..10].copy_from_slice(&self.epoch.to_be_bytes());
        out[10] = self.rights.tag();
        Ok(11)
    }
}

impl WirePayload for MemGrant {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 11 {
            return Err(CodecError::Invalid("memory grant carries eleven bytes"));
        }
        let mut ptr = [0u8; 4];
        let mut epoch = [0u8; 4];
        ptr.copy_from_slice(&bytes[1..5]);
        epoch.copy_from_slice(&bytes[6..10]);
        Ok(Self::new(
            bytes[0],
            u32::from_be_bytes(ptr),
            bytes[5],
            u32::from_be_bytes(epoch),
            MemRights::decode(bytes[10])?,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemRelease {
    lease_id: u8,
}

impl MemRelease {
    pub const fn new(lease_id: u8) -> Self {
        Self { lease_id }
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }
}

impl WireEncode for MemRelease {
    fn encoded_len(&self) -> Option<usize> {
        Some(1)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let Some(first) = out.first_mut() else {
            return Err(CodecError::Truncated);
        };
        *first = self.lease_id;
        Ok(1)
    }
}

impl WirePayload for MemRelease {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("memory release carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemCommit {
    lease_id: u8,
    written: u8,
}

impl MemCommit {
    pub const fn new(lease_id: u8, written: u8) -> Self {
        Self { lease_id, written }
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn written(&self) -> u8 {
        self.written
    }
}

impl WireEncode for MemCommit {
    fn encoded_len(&self) -> Option<usize> {
        Some(2)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 2 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.lease_id;
        out[1] = self.written;
        Ok(2)
    }
}

impl WirePayload for MemCommit {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("memory commit carries two bytes"));
        }
        Ok(Self::new(bytes[0], bytes[1]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemFenceReason {
    MemoryGrow,
    Trap,
    Suspend,
    Kill,
    HotSwap,
}

impl MemFenceReason {
    pub const fn tag(self) -> u8 {
        match self {
            Self::MemoryGrow => 1,
            Self::Trap => 2,
            Self::Suspend => 3,
            Self::Kill => 4,
            Self::HotSwap => 5,
        }
    }

    fn decode(tag: u8) -> Result<Self, CodecError> {
        match tag {
            1 => Ok(Self::MemoryGrow),
            2 => Ok(Self::Trap),
            3 => Ok(Self::Suspend),
            4 => Ok(Self::Kill),
            5 => Ok(Self::HotSwap),
            _ => Err(CodecError::Invalid("unknown memory fence reason")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemFence {
    reason: MemFenceReason,
    new_epoch: u32,
}

impl MemFence {
    pub const fn new(reason: MemFenceReason, new_epoch: u32) -> Self {
        Self { reason, new_epoch }
    }

    pub const fn reason(&self) -> MemFenceReason {
        self.reason
    }

    pub const fn new_epoch(&self) -> u32 {
        self.new_epoch
    }
}

impl WireEncode for MemFence {
    fn encoded_len(&self) -> Option<usize> {
        Some(5)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 5 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.reason.tag();
        out[1..5].copy_from_slice(&self.new_epoch.to_be_bytes());
        Ok(5)
    }
}

impl WirePayload for MemFence {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 5 {
            return Err(CodecError::Invalid("memory fence carries five bytes"));
        }
        let mut new_epoch = [0u8; 4];
        new_epoch.copy_from_slice(&bytes[1..5]);
        Ok(Self::new(
            MemFenceReason::decode(bytes[0])?,
            u32::from_be_bytes(new_epoch),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtImageBegin {
    slot: u8,
    total_len: u32,
    generation: u32,
}

impl MgmtImageBegin {
    pub const fn new(slot: u8, total_len: u32, generation: u32) -> Self {
        Self {
            slot,
            total_len,
            generation,
        }
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    pub const fn total_len(&self) -> u32 {
        self.total_len
    }

    pub const fn generation(&self) -> u32 {
        self.generation
    }
}

impl WireEncode for MgmtImageBegin {
    fn encoded_len(&self) -> Option<usize> {
        Some(9)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 9 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.slot;
        out[1..5].copy_from_slice(&self.total_len.to_be_bytes());
        out[5..9].copy_from_slice(&self.generation.to_be_bytes());
        Ok(9)
    }
}

impl WirePayload for MgmtImageBegin {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 9 {
            return Err(CodecError::Invalid("image begin carries nine bytes"));
        }
        let mut total_len = [0u8; 4];
        let mut generation = [0u8; 4];
        total_len.copy_from_slice(&bytes[1..5]);
        generation.copy_from_slice(&bytes[5..9]);
        Ok(Self::new(
            bytes[0],
            u32::from_be_bytes(total_len),
            u32::from_be_bytes(generation),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtImageChunk {
    slot: u8,
    offset: u32,
    len: u8,
    bytes: [u8; MGMT_IMAGE_CHUNK_CAPACITY],
}

impl MgmtImageChunk {
    pub fn new(slot: u8, offset: u32, bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() > MGMT_IMAGE_CHUNK_CAPACITY {
            return Err(CodecError::Invalid("image chunk exceeds fixed capacity"));
        }
        let mut out = [0u8; MGMT_IMAGE_CHUNK_CAPACITY];
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            slot,
            offset,
            len: bytes.len() as u8,
            bytes: out,
        })
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    pub const fn offset(&self) -> u32 {
        self.offset
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len()]
    }
}

impl WireEncode for MgmtImageChunk {
    fn encoded_len(&self) -> Option<usize> {
        Some(6 + self.len())
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let len = self.len();
        if out.len() < 6 + len {
            return Err(CodecError::Truncated);
        }
        out[0] = self.slot;
        out[1..5].copy_from_slice(&self.offset.to_be_bytes());
        out[5] = self.len;
        out[6..6 + len].copy_from_slice(self.as_bytes());
        Ok(6 + len)
    }
}

impl WirePayload for MgmtImageChunk {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() < 6 {
            return Err(CodecError::Truncated);
        }
        let len = bytes[5] as usize;
        if bytes.len() != 6 + len {
            return Err(CodecError::Invalid("image chunk length mismatch"));
        }
        let mut offset = [0u8; 4];
        offset.copy_from_slice(&bytes[1..5]);
        Self::new(bytes[0], u32::from_be_bytes(offset), &bytes[6..])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtImageEnd {
    slot: u8,
    expected_len: u32,
}

impl MgmtImageEnd {
    pub const fn new(slot: u8, expected_len: u32) -> Self {
        Self { slot, expected_len }
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    pub const fn expected_len(&self) -> u32 {
        self.expected_len
    }
}

impl WireEncode for MgmtImageEnd {
    fn encoded_len(&self) -> Option<usize> {
        Some(5)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 5 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.slot;
        out[1..5].copy_from_slice(&self.expected_len.to_be_bytes());
        Ok(5)
    }
}

impl WirePayload for MgmtImageEnd {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 5 {
            return Err(CodecError::Invalid("image end carries five bytes"));
        }
        let mut expected_len = [0u8; 4];
        expected_len.copy_from_slice(&bytes[1..5]);
        Ok(Self::new(bytes[0], u32::from_be_bytes(expected_len)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtImageActivate {
    slot: u8,
    fence_epoch: u32,
}

impl MgmtImageActivate {
    pub const fn new(slot: u8, fence_epoch: u32) -> Self {
        Self { slot, fence_epoch }
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    pub const fn fence_epoch(&self) -> u32 {
        self.fence_epoch
    }
}

impl WireEncode for MgmtImageActivate {
    fn encoded_len(&self) -> Option<usize> {
        Some(5)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 5 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.slot;
        out[1..5].copy_from_slice(&self.fence_epoch.to_be_bytes());
        Ok(5)
    }
}

impl WirePayload for MgmtImageActivate {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 5 {
            return Err(CodecError::Invalid("image activate carries five bytes"));
        }
        let mut fence_epoch = [0u8; 4];
        fence_epoch.copy_from_slice(&bytes[1..5]);
        Ok(Self::new(bytes[0], u32::from_be_bytes(fence_epoch)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtImageRollback {
    slot: u8,
}

impl MgmtImageRollback {
    pub const fn new(slot: u8) -> Self {
        Self { slot }
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }
}

impl WireEncode for MgmtImageRollback {
    fn encoded_len(&self) -> Option<usize> {
        Some(1)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let Some(first) = out.first_mut() else {
            return Err(CodecError::Truncated);
        };
        *first = self.slot;
        Ok(1)
    }
}

impl WirePayload for MgmtImageRollback {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("image rollback carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MgmtStatusCode {
    Ok,
    InvalidImage,
    NeedFence,
    BadSlot,
    RollbackEmpty,
    BadFenceEpoch,
    AuthFailed,
    BadSessionGeneration,
    ImageTooLarge,
    OffsetMismatch,
    LengthMismatch,
    BadChunkIndex,
}

impl MgmtStatusCode {
    pub const fn tag(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::InvalidImage => 1,
            Self::NeedFence => 2,
            Self::BadSlot => 3,
            Self::RollbackEmpty => 4,
            Self::BadFenceEpoch => 5,
            Self::AuthFailed => 6,
            Self::BadSessionGeneration => 7,
            Self::ImageTooLarge => 8,
            Self::OffsetMismatch => 9,
            Self::LengthMismatch => 10,
            Self::BadChunkIndex => 11,
        }
    }

    fn decode(tag: u8) -> Result<Self, CodecError> {
        match tag {
            0 => Ok(Self::Ok),
            1 => Ok(Self::InvalidImage),
            2 => Ok(Self::NeedFence),
            3 => Ok(Self::BadSlot),
            4 => Ok(Self::RollbackEmpty),
            5 => Ok(Self::BadFenceEpoch),
            6 => Ok(Self::AuthFailed),
            7 => Ok(Self::BadSessionGeneration),
            8 => Ok(Self::ImageTooLarge),
            9 => Ok(Self::OffsetMismatch),
            10 => Ok(Self::LengthMismatch),
            11 => Ok(Self::BadChunkIndex),
            _ => Err(CodecError::Invalid("unknown management status code")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtStatus {
    slot: u8,
    code: MgmtStatusCode,
}

impl MgmtStatus {
    pub const fn new(slot: u8, code: MgmtStatusCode) -> Self {
        Self { slot, code }
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    pub const fn code(&self) -> MgmtStatusCode {
        self.code
    }
}

impl WireEncode for MgmtStatus {
    fn encoded_len(&self) -> Option<usize> {
        Some(2)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 2 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.slot;
        out[1] = self.code.tag();
        Ok(2)
    }
}

impl WirePayload for MgmtStatus {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("management status carries two bytes"));
        }
        Ok(Self::new(bytes[0], MgmtStatusCode::decode(bytes[1])?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerSleepUntil {
    tick: u64,
}

impl TimerSleepUntil {
    pub const fn new(tick: u64) -> Self {
        Self { tick }
    }

    pub const fn tick(&self) -> u64 {
        self.tick
    }
}

impl WireEncode for TimerSleepUntil {
    fn encoded_len(&self) -> Option<usize> {
        Some(8)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 8 {
            return Err(CodecError::Truncated);
        }
        out[..8].copy_from_slice(&self.tick.to_be_bytes());
        Ok(8)
    }
}

impl WirePayload for TimerSleepUntil {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 8 {
            return Err(CodecError::Invalid(
                "timer sleep request carries eight bytes",
            ));
        }
        let mut tick = [0u8; 8];
        tick.copy_from_slice(bytes);
        Ok(Self::new(u64::from_be_bytes(tick)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerSleepDone {
    tick: u64,
}

impl TimerSleepDone {
    pub const fn new(tick: u64) -> Self {
        Self { tick }
    }

    pub const fn tick(&self) -> u64 {
        self.tick
    }
}

impl WireEncode for TimerSleepDone {
    fn encoded_len(&self) -> Option<usize> {
        Some(8)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 8 {
            return Err(CodecError::Truncated);
        }
        out[..8].copy_from_slice(&self.tick.to_be_bytes());
        Ok(8)
    }
}

impl WirePayload for TimerSleepDone {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 8 {
            return Err(CodecError::Invalid("timer sleep reply carries eight bytes"));
        }
        let mut tick = [0u8; 8];
        tick.copy_from_slice(bytes);
        Ok(Self::new(u64::from_be_bytes(tick)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpioSet {
    pin: u8,
    high: bool,
}

impl GpioSet {
    pub const fn new(pin: u8, high: bool) -> Self {
        Self { pin, high }
    }

    pub const fn pin(&self) -> u8 {
        self.pin
    }

    pub const fn high(&self) -> bool {
        self.high
    }

    pub const fn from_wasm_value(value: u32) -> Self {
        Self::new((value & 0xff) as u8, (value & 0x100) != 0)
    }
}

impl WireEncode for GpioSet {
    fn encoded_len(&self) -> Option<usize> {
        Some(2)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 2 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.pin;
        out[1] = self.high as u8;
        Ok(2)
    }
}

impl WirePayload for GpioSet {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("gpio set carries two bytes"));
        }
        match bytes[1] {
            0 => Ok(Self::new(bytes[0], false)),
            1 => Ok(Self::new(bytes[0], true)),
            _ => Err(CodecError::Invalid("gpio level must be 0 or 1")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpioWait {
    fd: u8,
    wait_id: u16,
    pin: u8,
    generation: u16,
}

impl GpioWait {
    pub const fn new(fd: u8, wait_id: u16, pin: u8, generation: u16) -> Self {
        Self {
            fd,
            wait_id,
            pin,
            generation,
        }
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    pub const fn wait_id(&self) -> u16 {
        self.wait_id
    }

    pub const fn pin(&self) -> u8 {
        self.pin
    }

    pub const fn generation(&self) -> u16 {
        self.generation
    }
}

impl WireEncode for GpioWait {
    fn encoded_len(&self) -> Option<usize> {
        Some(6)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 6 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.fd;
        out[1..3].copy_from_slice(&self.wait_id.to_be_bytes());
        out[3] = self.pin;
        out[4..6].copy_from_slice(&self.generation.to_be_bytes());
        Ok(6)
    }
}

impl WirePayload for GpioWait {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 6 {
            return Err(CodecError::Invalid("gpio wait carries six bytes"));
        }
        Ok(Self::new(
            bytes[0],
            u16::from_be_bytes([bytes[1], bytes[2]]),
            bytes[3],
            u16::from_be_bytes([bytes[4], bytes[5]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpioEdge {
    wait_id: u16,
    pin: u8,
    high: bool,
    generation: u16,
}

impl GpioEdge {
    pub const fn new(wait_id: u16, pin: u8, high: bool, generation: u16) -> Self {
        Self {
            wait_id,
            pin,
            high,
            generation,
        }
    }

    pub const fn wait_id(&self) -> u16 {
        self.wait_id
    }

    pub const fn pin(&self) -> u8 {
        self.pin
    }

    pub const fn high(&self) -> bool {
        self.high
    }

    pub const fn generation(&self) -> u16 {
        self.generation
    }
}

impl WireEncode for GpioEdge {
    fn encoded_len(&self) -> Option<usize> {
        Some(6)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 6 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.wait_id.to_be_bytes());
        out[2] = self.pin;
        out[3] = self.high as u8;
        out[4..6].copy_from_slice(&self.generation.to_be_bytes());
        Ok(6)
    }
}

impl WirePayload for GpioEdge {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 6 {
            return Err(CodecError::Invalid("gpio edge carries six bytes"));
        }
        let high = match bytes[3] {
            0 => false,
            1 => true,
            _ => return Err(CodecError::Invalid("gpio edge level must be 0 or 1")),
        };
        Ok(Self::new(
            u16::from_be_bytes([bytes[0], bytes[1]]),
            bytes[2],
            high,
            u16::from_be_bytes([bytes[4], bytes[5]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UartWrite {
    len: u8,
    bytes: [u8; UART_WRITE_CHUNK_CAPACITY],
}

impl UartWrite {
    pub fn new(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() > UART_WRITE_CHUNK_CAPACITY {
            return Err(CodecError::Invalid("uart write exceeds fixed capacity"));
        }
        let mut out = [0u8; UART_WRITE_CHUNK_CAPACITY];
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            len: bytes.len() as u8,
            bytes: out,
        })
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len()]
    }
}

impl WireEncode for UartWrite {
    fn encoded_len(&self) -> Option<usize> {
        Some(1 + self.len())
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let len = self.len();
        if out.len() < 1 + len {
            return Err(CodecError::Truncated);
        }
        out[0] = self.len;
        out[1..1 + len].copy_from_slice(self.as_bytes());
        Ok(1 + len)
    }
}

impl WirePayload for UartWrite {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        let Some((&len, payload)) = bytes.split_first() else {
            return Err(CodecError::Truncated);
        };
        if payload.len() != len as usize {
            return Err(CodecError::Invalid("uart write length mismatch"));
        }
        Self::new(payload)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UartWriteDone {
    written: u8,
}

impl UartWriteDone {
    pub const fn new(written: u8) -> Self {
        Self { written }
    }

    pub const fn written(&self) -> u8 {
        self.written
    }
}

impl WireEncode for UartWriteDone {
    fn encoded_len(&self) -> Option<usize> {
        Some(1)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let Some(first) = out.first_mut() else {
            return Err(CodecError::Truncated);
        };
        *first = self.written;
        Ok(1)
    }
}

impl WirePayload for UartWriteDone {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("uart write reply carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetRun {
    run_id: u16,
    generation: u16,
    fuel: u32,
    deadline_tick: u64,
}

impl BudgetRun {
    pub const fn new(run_id: u16, generation: u16, fuel: u32, deadline_tick: u64) -> Self {
        Self {
            run_id,
            generation,
            fuel,
            deadline_tick,
        }
    }

    pub const fn run_id(&self) -> u16 {
        self.run_id
    }

    pub const fn generation(&self) -> u16 {
        self.generation
    }

    pub const fn fuel(&self) -> u32 {
        self.fuel
    }

    pub const fn deadline_tick(&self) -> u64 {
        self.deadline_tick
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 16 {
            return Err(CodecError::Invalid("budget run carries sixteen bytes"));
        }
        Ok(Self::new(
            u16::from_be_bytes([bytes[0], bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
            u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            u64::from_be_bytes([
                bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ]),
        ))
    }
}

impl WireEncode for BudgetRun {
    fn encoded_len(&self) -> Option<usize> {
        Some(16)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 16 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.run_id.to_be_bytes());
        out[2..4].copy_from_slice(&self.generation.to_be_bytes());
        out[4..8].copy_from_slice(&self.fuel.to_be_bytes());
        out[8..16].copy_from_slice(&self.deadline_tick.to_be_bytes());
        Ok(16)
    }
}

impl WirePayload for BudgetRun {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        Self::decode(input.as_bytes())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetExpired {
    run_id: u16,
    generation: u16,
}

impl BudgetExpired {
    pub const fn new(run_id: u16, generation: u16) -> Self {
        Self { run_id, generation }
    }

    pub const fn run_id(&self) -> u16 {
        self.run_id
    }

    pub const fn generation(&self) -> u16 {
        self.generation
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 4 {
            return Err(CodecError::Invalid("budget event carries four bytes"));
        }
        Ok(Self::new(
            u16::from_be_bytes([bytes[0], bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ))
    }
}

impl WireEncode for BudgetExpired {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.run_id.to_be_bytes());
        out[2..4].copy_from_slice(&self.generation.to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for BudgetExpired {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        Self::decode(input.as_bytes())
    }
}

pub type BudgetSuspend = BudgetExpired;
pub type BudgetRestart = BudgetRun;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineReq {
    LogU32(u32),
    Yield,
    Wasip1Stdout(StdoutChunk),
    Wasip1Stderr(StderrChunk),
    Wasip1Stdin(StdinRequest),
    Wasip1ClockNow,
    Wasip1RandomSeed,
    Wasip1Exit(Wasip1ExitStatus),
    FdWrite(FdWrite),
    FdRead(FdRead),
    FdFdstatGet(FdRequest),
    FdClose(FdRequest),
    ClockResGet(ClockResGet),
    ClockTimeGet(ClockTimeGet),
    PollOneoff(PollOneoff),
    RandomGet(RandomGet),
    ProcExit(ProcExitStatus),
    ArgsGet(ArgsGet),
    EnvironGet(EnvironGet),
    TimerSleepUntil(TimerSleepUntil),
    GpioSet(GpioSet),
}

impl WireEncode for EngineReq {
    fn encoded_len(&self) -> Option<usize> {
        Some(match self {
            Self::LogU32(_) => 5,
            Self::Yield => 1,
            Self::Wasip1Stdout(chunk) => 3 + chunk.len(),
            Self::Wasip1Stderr(chunk) => 3 + chunk.len(),
            Self::Wasip1Stdin(_) => 3,
            Self::Wasip1ClockNow => 1,
            Self::Wasip1RandomSeed => 1,
            Self::Wasip1Exit(_) => 2,
            Self::FdWrite(write) => 4 + write.len(),
            Self::FdRead(_) => 4,
            Self::FdFdstatGet(_) => 2,
            Self::FdClose(_) => 2,
            Self::ClockResGet(_) => 2,
            Self::ClockTimeGet(_) => 10,
            Self::PollOneoff(_) => 9,
            Self::RandomGet(_) => 3,
            Self::ProcExit(_) => 2,
            Self::ArgsGet(_) => 3,
            Self::EnvironGet(_) => 3,
            Self::TimerSleepUntil(_) => 9,
            Self::GpioSet(_) => 3,
        })
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        match *self {
            Self::LogU32(value) => {
                if out.len() < 5 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_LOG_U32;
                out[1..5].copy_from_slice(&value.to_be_bytes());
                Ok(5)
            }
            Self::Yield => {
                if out.is_empty() {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_YIELD;
                Ok(1)
            }
            Self::Wasip1Stdout(chunk) => {
                let len = chunk.len();
                if out.len() < 3 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASIP1_STDOUT;
                out[1] = chunk.lease_id();
                out[2] = len as u8;
                out[3..3 + len].copy_from_slice(chunk.as_bytes());
                Ok(3 + len)
            }
            Self::Wasip1Stderr(chunk) => {
                let len = chunk.len();
                if out.len() < 3 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASIP1_STDERR;
                out[1] = chunk.lease_id();
                out[2] = len as u8;
                out[3..3 + len].copy_from_slice(chunk.as_bytes());
                Ok(3 + len)
            }
            Self::Wasip1Stdin(request) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASIP1_STDIN;
                out[1] = request.lease_id();
                out[2] = request.max_len();
                Ok(3)
            }
            Self::Wasip1ClockNow => {
                if out.is_empty() {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASIP1_CLOCK_NOW;
                Ok(1)
            }
            Self::Wasip1RandomSeed => {
                if out.is_empty() {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASIP1_RANDOM_SEED;
                Ok(1)
            }
            Self::Wasip1Exit(status) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASIP1_EXIT;
                out[1] = status.code();
                Ok(2)
            }
            Self::FdWrite(write) => {
                let len = write.len();
                if out.len() < 4 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_FD_WRITE;
                out[1] = write.fd();
                out[2] = write.lease_id();
                out[3] = len as u8;
                out[4..4 + len].copy_from_slice(write.as_bytes());
                Ok(4 + len)
            }
            Self::FdRead(read) => {
                if out.len() < 4 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_FD_READ;
                out[1] = read.fd();
                out[2] = read.lease_id();
                out[3] = read.max_len();
                Ok(4)
            }
            Self::FdFdstatGet(request) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_FD_FDSTAT_GET;
                out[1] = request.fd();
                Ok(2)
            }
            Self::FdClose(request) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_FD_CLOSE;
                out[1] = request.fd();
                Ok(2)
            }
            Self::ClockResGet(request) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_CLOCK_RES_GET;
                out[1] = request.clock_id();
                Ok(2)
            }
            Self::ClockTimeGet(request) => {
                if out.len() < 10 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_CLOCK_TIME_GET;
                out[1] = request.clock_id();
                out[2..10].copy_from_slice(&request.precision().to_be_bytes());
                Ok(10)
            }
            Self::PollOneoff(request) => {
                if out.len() < 9 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_POLL_ONEOFF;
                out[1..9].copy_from_slice(&request.timeout_tick().to_be_bytes());
                Ok(9)
            }
            Self::RandomGet(request) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_RANDOM_GET;
                out[1] = request.lease_id();
                out[2] = request.max_len();
                Ok(3)
            }
            Self::ProcExit(status) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_PROC_EXIT;
                out[1] = status.code();
                Ok(2)
            }
            Self::ArgsGet(request) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_ARGS_GET;
                out[1] = request.lease_id();
                out[2] = request.max_len();
                Ok(3)
            }
            Self::EnvironGet(request) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_WASI_ENVIRON_GET;
                out[1] = request.lease_id();
                out[2] = request.max_len();
                Ok(3)
            }
            Self::TimerSleepUntil(sleep) => {
                if out.len() < 9 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_TIMER_SLEEP_UNTIL;
                out[1..9].copy_from_slice(&sleep.tick().to_be_bytes());
                Ok(9)
            }
            Self::GpioSet(set) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_REQ_GPIO_SET;
                out[1] = set.pin();
                out[2] = set.high() as u8;
                Ok(3)
            }
        }
    }
}

impl WirePayload for EngineReq {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        let Some((&tag, rest)) = bytes.split_first() else {
            return Err(CodecError::Truncated);
        };
        match tag {
            TAG_REQ_LOG_U32 => Ok(Self::LogU32(decode_u32_payload(rest)?)),
            TAG_REQ_YIELD => {
                if !rest.is_empty() {
                    return Err(CodecError::Invalid("yield request carries no payload"));
                }
                Ok(Self::Yield)
            }
            TAG_REQ_WASIP1_STDOUT => Ok(Self::Wasip1Stdout(StdoutChunk::decode(rest)?)),
            TAG_REQ_WASIP1_STDERR => Ok(Self::Wasip1Stderr(StderrChunk::decode(rest)?)),
            TAG_REQ_WASIP1_STDIN => Ok(Self::Wasip1Stdin(StdinRequest::decode(rest)?)),
            TAG_REQ_WASIP1_CLOCK_NOW => {
                if !rest.is_empty() {
                    return Err(CodecError::Invalid("clock request carries no payload"));
                }
                Ok(Self::Wasip1ClockNow)
            }
            TAG_REQ_WASIP1_RANDOM_SEED => {
                if !rest.is_empty() {
                    return Err(CodecError::Invalid("random request carries no payload"));
                }
                Ok(Self::Wasip1RandomSeed)
            }
            TAG_REQ_WASIP1_EXIT => Ok(Self::Wasip1Exit(Wasip1ExitStatus::decode(rest)?)),
            TAG_REQ_WASI_FD_WRITE => Ok(Self::FdWrite(FdWrite::decode(rest)?)),
            TAG_REQ_WASI_FD_READ => Ok(Self::FdRead(FdRead::decode(rest)?)),
            TAG_REQ_WASI_FD_FDSTAT_GET => Ok(Self::FdFdstatGet(FdRequest::decode(rest)?)),
            TAG_REQ_WASI_FD_CLOSE => Ok(Self::FdClose(FdRequest::decode(rest)?)),
            TAG_REQ_WASI_CLOCK_RES_GET => Ok(Self::ClockResGet(ClockResGet::decode(rest)?)),
            TAG_REQ_WASI_CLOCK_TIME_GET => Ok(Self::ClockTimeGet(ClockTimeGet::decode(rest)?)),
            TAG_REQ_WASI_POLL_ONEOFF => Ok(Self::PollOneoff(PollOneoff::decode(rest)?)),
            TAG_REQ_WASI_RANDOM_GET => Ok(Self::RandomGet(RandomGet::decode(rest)?)),
            TAG_REQ_WASI_PROC_EXIT => Ok(Self::ProcExit(ProcExitStatus::decode(rest)?)),
            TAG_REQ_WASI_ARGS_GET => Ok(Self::ArgsGet(ArgsGet::decode(rest)?)),
            TAG_REQ_WASI_ENVIRON_GET => Ok(Self::EnvironGet(EnvironGet::decode(rest)?)),
            TAG_REQ_TIMER_SLEEP_UNTIL => Ok(Self::TimerSleepUntil(
                TimerSleepUntil::decode_payload(Payload::new(rest))?,
            )),
            TAG_REQ_GPIO_SET => Ok(Self::GpioSet(GpioSet::decode_payload(Payload::new(rest))?)),
            _ => Err(CodecError::Invalid("unknown engine request tag")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineRet {
    Logged(u32),
    Yielded,
    Wasip1StdoutWritten(u8),
    Wasip1StderrWritten(u8),
    Wasip1StdinRead(StdinChunk),
    Wasip1ClockNow(ClockNow),
    Wasip1RandomSeed(RandomSeed),
    FdWriteDone(FdWriteDone),
    FdReadDone(FdReadDone),
    FdStat(FdStat),
    FdClosed(FdClosed),
    ClockResolution(ClockResolution),
    ClockTime(ClockNow),
    PollReady(PollReady),
    RandomDone(RandomDone),
    ArgsDone(ArgsDone),
    EnvironDone(EnvironDone),
    TimerSleepDone(TimerSleepDone),
    GpioSetDone(GpioSet),
}

impl WireEncode for EngineRet {
    fn encoded_len(&self) -> Option<usize> {
        Some(match self {
            Self::Logged(_) => 5,
            Self::Yielded => 1,
            Self::Wasip1StdoutWritten(_) => 2,
            Self::Wasip1StderrWritten(_) => 2,
            Self::Wasip1StdinRead(chunk) => 3 + chunk.len(),
            Self::Wasip1ClockNow(_) => 9,
            Self::Wasip1RandomSeed(_) => 17,
            Self::FdWriteDone(_) => 3,
            Self::FdReadDone(done) => 4 + done.len(),
            Self::FdStat(_) => 3,
            Self::FdClosed(_) => 2,
            Self::ClockResolution(_) => 9,
            Self::ClockTime(_) => 9,
            Self::PollReady(_) => 2,
            Self::RandomDone(done) => 3 + done.len(),
            Self::ArgsDone(done) => 3 + done.len(),
            Self::EnvironDone(done) => 3 + done.len(),
            Self::TimerSleepDone(_) => 9,
            Self::GpioSetDone(_) => 3,
        })
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        match *self {
            Self::Logged(value) => {
                if out.len() < 5 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_LOGGED;
                out[1..5].copy_from_slice(&value.to_be_bytes());
                Ok(5)
            }
            Self::Yielded => {
                if out.is_empty() {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_YIELDED;
                Ok(1)
            }
            Self::Wasip1StdoutWritten(written) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASIP1_STDOUT_WRITTEN;
                out[1] = written;
                Ok(2)
            }
            Self::Wasip1StderrWritten(written) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASIP1_STDERR_WRITTEN;
                out[1] = written;
                Ok(2)
            }
            Self::Wasip1StdinRead(chunk) => {
                let len = chunk.len();
                if out.len() < 3 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASIP1_STDIN_READ;
                out[1] = chunk.lease_id();
                out[2] = len as u8;
                out[3..3 + len].copy_from_slice(chunk.as_bytes());
                Ok(3 + len)
            }
            Self::Wasip1ClockNow(now) => {
                if out.len() < 9 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASIP1_CLOCK_NOW;
                out[1..9].copy_from_slice(&now.nanos().to_be_bytes());
                Ok(9)
            }
            Self::Wasip1RandomSeed(seed) => {
                if out.len() < 17 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASIP1_RANDOM_SEED;
                out[1..9].copy_from_slice(&seed.lo().to_be_bytes());
                out[9..17].copy_from_slice(&seed.hi().to_be_bytes());
                Ok(17)
            }
            Self::FdWriteDone(done) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_FD_WRITE_DONE;
                out[1] = done.fd();
                out[2] = done.written();
                Ok(3)
            }
            Self::FdReadDone(done) => {
                let len = done.len();
                if out.len() < 4 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_FD_READ_DONE;
                out[1] = done.fd();
                out[2] = done.lease_id();
                out[3] = len as u8;
                out[4..4 + len].copy_from_slice(done.as_bytes());
                Ok(4 + len)
            }
            Self::FdStat(stat) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_FD_FDSTAT;
                out[1] = stat.fd();
                out[2] = stat.rights().tag();
                Ok(3)
            }
            Self::FdClosed(closed) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_FD_CLOSED;
                out[1] = closed.fd();
                Ok(2)
            }
            Self::ClockResolution(resolution) => {
                if out.len() < 9 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_CLOCK_RESOLUTION;
                out[1..9].copy_from_slice(&resolution.nanos().to_be_bytes());
                Ok(9)
            }
            Self::ClockTime(now) => {
                if out.len() < 9 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_CLOCK_TIME;
                out[1..9].copy_from_slice(&now.nanos().to_be_bytes());
                Ok(9)
            }
            Self::PollReady(ready) => {
                if out.len() < 2 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_POLL_READY;
                out[1] = ready.ready();
                Ok(2)
            }
            Self::RandomDone(done) => {
                let len = done.len();
                if out.len() < 3 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_RANDOM_DONE;
                out[1] = done.lease_id();
                out[2] = len as u8;
                out[3..3 + len].copy_from_slice(done.as_bytes());
                Ok(3 + len)
            }
            Self::ArgsDone(done) => {
                let len = done.len();
                if out.len() < 3 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_ARGS_DONE;
                out[1] = done.lease_id();
                out[2] = len as u8;
                out[3..3 + len].copy_from_slice(done.as_bytes());
                Ok(3 + len)
            }
            Self::EnvironDone(done) => {
                let len = done.len();
                if out.len() < 3 + len {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_WASI_ENVIRON_DONE;
                out[1] = done.lease_id();
                out[2] = len as u8;
                out[3..3 + len].copy_from_slice(done.as_bytes());
                Ok(3 + len)
            }
            Self::TimerSleepDone(done) => {
                if out.len() < 9 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_TIMER_SLEEP_DONE;
                out[1..9].copy_from_slice(&done.tick().to_be_bytes());
                Ok(9)
            }
            Self::GpioSetDone(set) => {
                if out.len() < 3 {
                    return Err(CodecError::Truncated);
                }
                out[0] = TAG_RET_GPIO_SET_DONE;
                out[1] = set.pin();
                out[2] = set.high() as u8;
                Ok(3)
            }
        }
    }
}

impl WirePayload for EngineRet {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        let Some((&tag, rest)) = bytes.split_first() else {
            return Err(CodecError::Truncated);
        };
        match tag {
            TAG_RET_LOGGED => Ok(Self::Logged(decode_u32_payload(rest)?)),
            TAG_RET_YIELDED => {
                if !rest.is_empty() {
                    return Err(CodecError::Invalid("yield reply carries no payload"));
                }
                Ok(Self::Yielded)
            }
            TAG_RET_WASIP1_STDOUT_WRITTEN => {
                if rest.len() != 1 {
                    return Err(CodecError::Invalid("stdout reply carries one byte"));
                }
                Ok(Self::Wasip1StdoutWritten(rest[0]))
            }
            TAG_RET_WASIP1_STDERR_WRITTEN => {
                if rest.len() != 1 {
                    return Err(CodecError::Invalid("stderr reply carries one byte"));
                }
                Ok(Self::Wasip1StderrWritten(rest[0]))
            }
            TAG_RET_WASIP1_STDIN_READ => Ok(Self::Wasip1StdinRead(StdinChunk::decode(rest)?)),
            TAG_RET_WASIP1_CLOCK_NOW => Ok(Self::Wasip1ClockNow(ClockNow::decode(rest)?)),
            TAG_RET_WASIP1_RANDOM_SEED => Ok(Self::Wasip1RandomSeed(RandomSeed::decode(rest)?)),
            TAG_RET_WASI_FD_WRITE_DONE => Ok(Self::FdWriteDone(FdWriteDone::decode(rest)?)),
            TAG_RET_WASI_FD_READ_DONE => Ok(Self::FdReadDone(FdReadDone::decode(rest)?)),
            TAG_RET_WASI_FD_FDSTAT => Ok(Self::FdStat(FdStat::decode(rest)?)),
            TAG_RET_WASI_FD_CLOSED => Ok(Self::FdClosed(FdClosed::decode(rest)?)),
            TAG_RET_WASI_CLOCK_RESOLUTION => {
                Ok(Self::ClockResolution(ClockResolution::decode(rest)?))
            }
            TAG_RET_WASI_CLOCK_TIME => Ok(Self::ClockTime(ClockNow::decode(rest)?)),
            TAG_RET_WASI_POLL_READY => Ok(Self::PollReady(PollReady::decode(rest)?)),
            TAG_RET_WASI_RANDOM_DONE => Ok(Self::RandomDone(RandomDone::decode(rest)?)),
            TAG_RET_WASI_ARGS_DONE => Ok(Self::ArgsDone(ArgsDone::decode(rest)?)),
            TAG_RET_WASI_ENVIRON_DONE => Ok(Self::EnvironDone(EnvironDone::decode(rest)?)),
            TAG_RET_TIMER_SLEEP_DONE => Ok(Self::TimerSleepDone(TimerSleepDone::decode_payload(
                Payload::new(rest),
            )?)),
            TAG_RET_GPIO_SET_DONE => Ok(Self::GpioSetDone(GpioSet::decode_payload(Payload::new(
                rest,
            ))?)),
            _ => Err(CodecError::Invalid("unknown engine reply tag")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockNow {
    nanos: u64,
}

impl ClockNow {
    pub const fn new(nanos: u64) -> Self {
        Self { nanos }
    }

    pub const fn nanos(&self) -> u64 {
        self.nanos
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 8 {
            return Err(CodecError::Invalid("clock reply carries eight bytes"));
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(Self::new(u64::from_be_bytes(buf)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wasip1ExitStatus {
    code: u8,
}

impl Wasip1ExitStatus {
    pub const fn new(code: u8) -> Self {
        Self { code }
    }

    pub const fn code(&self) -> u8 {
        self.code
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("exit request carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomSeed {
    lo: u64,
    hi: u64,
}

impl RandomSeed {
    pub const fn new(lo: u64, hi: u64) -> Self {
        Self { lo, hi }
    }

    pub const fn lo(&self) -> u64 {
        self.lo
    }

    pub const fn hi(&self) -> u64 {
        self.hi
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 16 {
            return Err(CodecError::Invalid(
                "random seed reply carries sixteen bytes",
            ));
        }
        let mut lo = [0u8; 8];
        let mut hi = [0u8; 8];
        lo.copy_from_slice(&bytes[..8]);
        hi.copy_from_slice(&bytes[8..16]);
        Ok(Self::new(u64::from_be_bytes(lo), u64::from_be_bytes(hi)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StdinRequest {
    lease_id: u8,
    max_len: u8,
}

impl StdinRequest {
    pub fn new(max_len: u8) -> Result<Self, CodecError> {
        Self::new_with_lease(MEM_LEASE_NONE, max_len)
    }

    pub fn new_with_lease(lease_id: u8, max_len: u8) -> Result<Self, CodecError> {
        if max_len as usize > STDIN_CHUNK_CAPACITY {
            return Err(CodecError::Invalid("stdin request exceeds fixed capacity"));
        }
        Ok(Self { lease_id, max_len })
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn max_len(&self) -> u8 {
        self.max_len
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("stdin request carries two bytes"));
        }
        Self::new_with_lease(bytes[0], bytes[1])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wasip1StreamChunk {
    lease_id: u8,
    len: u8,
    bytes: [u8; WASIP1_STREAM_CHUNK_CAPACITY],
}

pub type StdoutChunk = Wasip1StreamChunk;
pub type StderrChunk = Wasip1StreamChunk;
pub type StdinChunk = Wasip1StreamChunk;

impl Wasip1StreamChunk {
    pub fn new(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::new_with_lease(MEM_LEASE_NONE, bytes)
    }

    pub fn new_with_lease(lease_id: u8, bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(CodecError::Invalid("stream chunk exceeds fixed capacity"));
        }
        let mut out = [0u8; WASIP1_STREAM_CHUNK_CAPACITY];
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            lease_id,
            len: bytes.len() as u8,
            bytes: out,
        })
    }

    pub fn with_lease(&self, lease_id: u8) -> Self {
        Self {
            lease_id,
            len: self.len,
            bytes: self.bytes,
        }
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len()]
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() < 2 {
            return Err(CodecError::Truncated);
        };
        let lease_id = bytes[0];
        let len = bytes[1] as usize;
        let payload = &bytes[2..];
        if len > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(CodecError::Invalid("stream chunk length exceeds capacity"));
        }
        if payload.len() != len {
            return Err(CodecError::Invalid("stream chunk length mismatch"));
        }
        Self::new_with_lease(lease_id, payload)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdWrite {
    fd: u8,
    chunk: Wasip1StreamChunk,
}

impl FdWrite {
    pub fn new(fd: u8, bytes: &[u8]) -> Result<Self, CodecError> {
        Self::new_with_lease(fd, MEM_LEASE_NONE, bytes)
    }

    pub fn new_with_lease(fd: u8, lease_id: u8, bytes: &[u8]) -> Result<Self, CodecError> {
        Ok(Self {
            fd,
            chunk: Wasip1StreamChunk::new_with_lease(lease_id, bytes)?,
        })
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    pub const fn lease_id(&self) -> u8 {
        self.chunk.lease_id()
    }

    pub const fn len(&self) -> usize {
        self.chunk.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.chunk.as_bytes()
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() < 3 {
            return Err(CodecError::Truncated);
        }
        let fd = bytes[0];
        Self::new_with_lease(fd, bytes[1], &bytes[3..]).and_then(|write| {
            if write.len() != bytes[2] as usize {
                return Err(CodecError::Invalid("fd_write length mismatch"));
            }
            Ok(write)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdRead {
    fd: u8,
    lease_id: u8,
    max_len: u8,
}

impl FdRead {
    pub fn new(fd: u8, max_len: u8) -> Result<Self, CodecError> {
        Self::new_with_lease(fd, MEM_LEASE_NONE, max_len)
    }

    pub fn new_with_lease(fd: u8, lease_id: u8, max_len: u8) -> Result<Self, CodecError> {
        if max_len as usize > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(CodecError::Invalid(
                "fd_read request exceeds fixed capacity",
            ));
        }
        Ok(Self {
            fd,
            lease_id,
            max_len,
        })
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn max_len(&self) -> u8 {
        self.max_len
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 3 {
            return Err(CodecError::Invalid("fd_read request carries three bytes"));
        }
        Self::new_with_lease(bytes[0], bytes[1], bytes[2])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdRequest {
    fd: u8,
}

impl FdRequest {
    pub const fn new(fd: u8) -> Self {
        Self { fd }
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("fd request carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdError {
    fd: u8,
    errno: u16,
}

impl FdError {
    pub const fn new(fd: u8, errno: u16) -> Self {
        Self { fd, errno }
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    pub const fn errno(&self) -> u16 {
        self.errno
    }
}

impl WireEncode for FdError {
    fn encoded_len(&self) -> Option<usize> {
        Some(3)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 3 {
            return Err(CodecError::Truncated);
        }
        out[0] = self.fd;
        out[1..3].copy_from_slice(&self.errno.to_be_bytes());
        Ok(3)
    }
}

impl WirePayload for FdError {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 3 {
            return Err(CodecError::Invalid("fd error carries three bytes"));
        }
        Ok(Self::new(
            bytes[0],
            u16::from_be_bytes([bytes[1], bytes[2]]),
        ))
    }
}

pub type FdErrorMsg = Msg<LABEL_WASI_FD_ERROR, FdError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockResGet {
    clock_id: u8,
}

impl ClockResGet {
    pub const fn new(clock_id: u8) -> Self {
        Self { clock_id }
    }

    pub const fn clock_id(&self) -> u8 {
        self.clock_id
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("clock_res_get carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTimeGet {
    clock_id: u8,
    precision: u64,
}

impl ClockTimeGet {
    pub const fn new(clock_id: u8, precision: u64) -> Self {
        Self {
            clock_id,
            precision,
        }
    }

    pub const fn clock_id(&self) -> u8 {
        self.clock_id
    }

    pub const fn precision(&self) -> u64 {
        self.precision
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 9 {
            return Err(CodecError::Invalid("clock_time_get carries nine bytes"));
        }
        let mut precision = [0u8; 8];
        precision.copy_from_slice(&bytes[1..9]);
        Ok(Self::new(bytes[0], u64::from_be_bytes(precision)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockResolution {
    nanos: u64,
}

impl ClockResolution {
    pub const fn new(nanos: u64) -> Self {
        Self { nanos }
    }

    pub const fn nanos(&self) -> u64 {
        self.nanos
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 8 {
            return Err(CodecError::Invalid("clock resolution carries eight bytes"));
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(Self::new(u64::from_be_bytes(buf)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PollOneoff {
    timeout_tick: u64,
}

impl PollOneoff {
    pub const fn new(timeout_tick: u64) -> Self {
        Self { timeout_tick }
    }

    pub const fn timeout_tick(&self) -> u64 {
        self.timeout_tick
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 8 {
            return Err(CodecError::Invalid("poll_oneoff carries eight bytes"));
        }
        let mut timeout = [0u8; 8];
        timeout.copy_from_slice(bytes);
        Ok(Self::new(u64::from_be_bytes(timeout)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomGet {
    lease_id: u8,
    max_len: u8,
}

impl RandomGet {
    pub fn new(max_len: u8) -> Result<Self, CodecError> {
        Self::new_with_lease(MEM_LEASE_NONE, max_len)
    }

    pub fn new_with_lease(lease_id: u8, max_len: u8) -> Result<Self, CodecError> {
        if max_len as usize > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(CodecError::Invalid(
                "random_get request exceeds fixed capacity",
            ));
        }
        Ok(Self { lease_id, max_len })
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn max_len(&self) -> u8 {
        self.max_len
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("random_get carries two bytes"));
        }
        Self::new_with_lease(bytes[0], bytes[1])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcExitStatus {
    code: u8,
}

impl ProcExitStatus {
    pub const fn new(code: u8) -> Self {
        Self { code }
    }

    pub const fn code(&self) -> u8 {
        self.code
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("proc_exit carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgsGet {
    lease_id: u8,
    max_len: u8,
}

impl ArgsGet {
    pub fn new(max_len: u8) -> Result<Self, CodecError> {
        Self::new_with_lease(MEM_LEASE_NONE, max_len)
    }

    pub fn new_with_lease(lease_id: u8, max_len: u8) -> Result<Self, CodecError> {
        if max_len as usize > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(CodecError::Invalid(
                "args_get request exceeds fixed capacity",
            ));
        }
        Ok(Self { lease_id, max_len })
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn max_len(&self) -> u8 {
        self.max_len
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("args_get carries two bytes"));
        }
        Self::new_with_lease(bytes[0], bytes[1])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnvironGet {
    lease_id: u8,
    max_len: u8,
}

impl EnvironGet {
    pub fn new(max_len: u8) -> Result<Self, CodecError> {
        Self::new_with_lease(MEM_LEASE_NONE, max_len)
    }

    pub fn new_with_lease(lease_id: u8, max_len: u8) -> Result<Self, CodecError> {
        if max_len as usize > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(CodecError::Invalid(
                "environ_get request exceeds fixed capacity",
            ));
        }
        Ok(Self { lease_id, max_len })
    }

    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn max_len(&self) -> u8 {
        self.max_len
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("environ_get carries two bytes"));
        }
        Self::new_with_lease(bytes[0], bytes[1])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdWriteDone {
    fd: u8,
    written: u8,
}

impl FdWriteDone {
    pub const fn new(fd: u8, written: u8) -> Self {
        Self { fd, written }
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    pub const fn written(&self) -> u8 {
        self.written
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("fd_write reply carries two bytes"));
        }
        Ok(Self::new(bytes[0], bytes[1]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdReadDone {
    fd: u8,
    chunk: Wasip1StreamChunk,
}

impl FdReadDone {
    pub fn new_with_lease(fd: u8, lease_id: u8, bytes: &[u8]) -> Result<Self, CodecError> {
        Ok(Self {
            fd,
            chunk: Wasip1StreamChunk::new_with_lease(lease_id, bytes)?,
        })
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    pub const fn lease_id(&self) -> u8 {
        self.chunk.lease_id()
    }

    pub const fn len(&self) -> usize {
        self.chunk.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.chunk.as_bytes()
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() < 3 {
            return Err(CodecError::Truncated);
        }
        let fd = bytes[0];
        Self::new_with_lease(fd, bytes[1], &bytes[3..]).and_then(|read| {
            if read.len() != bytes[2] as usize {
                return Err(CodecError::Invalid("fd_read reply length mismatch"));
            }
            Ok(read)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdStat {
    fd: u8,
    rights: MemRights,
}

impl FdStat {
    pub const fn new(fd: u8, rights: MemRights) -> Self {
        Self { fd, rights }
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    pub const fn rights(&self) -> MemRights {
        self.rights
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 2 {
            return Err(CodecError::Invalid("fd_fdstat_get reply carries two bytes"));
        }
        Ok(Self::new(bytes[0], MemRights::decode(bytes[1])?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdClosed {
    fd: u8,
}

impl FdClosed {
    pub const fn new(fd: u8) -> Self {
        Self { fd }
    }

    pub const fn fd(&self) -> u8 {
        self.fd
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("fd_close reply carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PollReady {
    ready: u8,
}

impl PollReady {
    pub const fn new(ready: u8) -> Self {
        Self { ready }
    }

    pub const fn ready(&self) -> u8 {
        self.ready
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != 1 {
            return Err(CodecError::Invalid("poll_oneoff reply carries one byte"));
        }
        Ok(Self::new(bytes[0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomDone {
    chunk: Wasip1StreamChunk,
}

impl RandomDone {
    pub fn new_with_lease(lease_id: u8, bytes: &[u8]) -> Result<Self, CodecError> {
        Ok(Self {
            chunk: Wasip1StreamChunk::new_with_lease(lease_id, bytes)?,
        })
    }

    pub const fn lease_id(&self) -> u8 {
        self.chunk.lease_id()
    }

    pub const fn len(&self) -> usize {
        self.chunk.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.chunk.as_bytes()
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        Wasip1StreamChunk::decode(bytes).map(|chunk| Self { chunk })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgsDone {
    chunk: Wasip1StreamChunk,
}

impl ArgsDone {
    pub fn new_with_lease(lease_id: u8, bytes: &[u8]) -> Result<Self, CodecError> {
        Ok(Self {
            chunk: Wasip1StreamChunk::new_with_lease(lease_id, bytes)?,
        })
    }

    pub const fn lease_id(&self) -> u8 {
        self.chunk.lease_id()
    }

    pub const fn len(&self) -> usize {
        self.chunk.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.chunk.as_bytes()
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        Wasip1StreamChunk::decode(bytes).map(|chunk| Self { chunk })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnvironDone {
    chunk: Wasip1StreamChunk,
}

impl EnvironDone {
    pub fn new_with_lease(lease_id: u8, bytes: &[u8]) -> Result<Self, CodecError> {
        Ok(Self {
            chunk: Wasip1StreamChunk::new_with_lease(lease_id, bytes)?,
        })
    }

    pub const fn lease_id(&self) -> u8 {
        self.chunk.lease_id()
    }

    pub const fn len(&self) -> usize {
        self.chunk.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.chunk.as_bytes()
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        Wasip1StreamChunk::decode(bytes).map(|chunk| Self { chunk })
    }
}

fn decode_u32_payload(bytes: &[u8]) -> Result<u32, CodecError> {
    if bytes.len() < 4 {
        return Err(CodecError::Truncated);
    }
    if bytes.len() > 4 {
        return Err(CodecError::Invalid("unexpected trailing payload bytes"));
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(bytes);
    Ok(u32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::{
        ArgsDone, ArgsGet, BudgetExpired, BudgetRestart, BudgetRun, BudgetSuspend, ClockNow,
        ClockResGet, ClockResolution, ClockTimeGet, EngineReq, EngineRet, EnvironDone, EnvironGet,
        FdClosed, FdRead, FdReadDone, FdRequest, FdStat, FdWrite, FdWriteDone, GpioEdge, GpioSet,
        GpioWait, MGMT_IMAGE_CHUNK_CAPACITY, MemBorrow, MemCommit, MemFence, MemFenceReason,
        MemGrant, MemRelease, MemRights, MgmtImageActivate, MgmtImageBegin, MgmtImageChunk,
        MgmtImageEnd, MgmtImageRollback, MgmtStatus, MgmtStatusCode, PollOneoff, PollReady,
        ProcExitStatus, RandomDone, RandomGet, RandomSeed, StderrChunk, StdinChunk, StdinRequest,
        StdoutChunk, TimerSleepDone, TimerSleepUntil, UartWrite, UartWriteDone, Wasip1ExitStatus,
    };
    use hibana::substrate::{
        cap::{ControlResourceKind, advanced::ScopeId},
        ids::{Lane, SessionId},
        wire::{CodecError, Payload, WireEncode, WirePayload},
    };

    fn encode<T: WireEncode>(value: &T, out: &mut [u8]) -> usize {
        value.encode_into(out).expect("encode payload")
    }

    #[test]
    fn plain_payload_labels_avoid_hibana_reserved_control_labels() {
        let labels = [
            super::LABEL_WASI_FD_WRITE,
            super::LABEL_WASI_FD_WRITE_RET,
            super::LABEL_WASI_FD_READ,
            super::LABEL_WASI_FD_READ_RET,
            super::LABEL_WASI_FD_FDSTAT_GET,
            super::LABEL_WASI_FD_FDSTAT_GET_RET,
            super::LABEL_WASI_FD_CLOSE,
            super::LABEL_WASI_FD_CLOSE_RET,
            super::LABEL_WASI_CLOCK_RES_GET,
            super::LABEL_WASI_CLOCK_RES_GET_RET,
            super::LABEL_WASI_CLOCK_TIME_GET,
            super::LABEL_WASI_CLOCK_TIME_GET_RET,
            super::LABEL_WASI_POLL_ONEOFF,
            super::LABEL_WASI_POLL_ONEOFF_RET,
            super::LABEL_WASI_RANDOM_GET,
            super::LABEL_WASI_RANDOM_GET_RET,
            super::LABEL_WASI_PROC_EXIT,
            super::LABEL_WASI_ARGS_GET,
            super::LABEL_WASI_ARGS_GET_RET,
            super::LABEL_WASI_ENVIRON_GET,
            super::LABEL_WASI_ENVIRON_GET_RET,
            super::LABEL_ENGINE_RUN,
            super::LABEL_ENGINE_BUDGET_EXPIRED,
            super::LABEL_ENGINE_SUSPEND,
            super::LABEL_ENGINE_RESTART,
            super::LABEL_GPIO_WAIT,
            super::LABEL_GPIO_SUBSCRIBE,
            super::LABEL_GPIO_EDGE,
            super::LABEL_GPIO_WAIT_RET,
            super::LABEL_UART_WRITE,
            super::LABEL_UART_WRITE_RET,
            super::LABEL_NET_STREAM_WRITE,
            super::LABEL_NET_STREAM_ACK,
            super::LABEL_NET_STREAM_READ,
            super::LABEL_NET_STREAM_READ_RET,
        ];
        for label in labels {
            assert_ne!(label, 48);
            assert_ne!(label, 49);
            assert_ne!(label, 57);
            assert!(label < super::LABEL_MEM_GRANT_READ_CONTROL);
        }
    }

    #[test]
    fn route_control_arm_ids_are_distinct_and_scope_preserving() {
        let scope = ScopeId::route(42);
        let sid = SessionId::new(7);
        let lane = Lane::new(17);

        let remote_sensor =
            <super::RemoteSensorRouteKind as ControlResourceKind>::mint_handle(sid, lane, scope);
        let remote_actuator =
            <super::RemoteActuatorRouteKind as ControlResourceKind>::mint_handle(sid, lane, scope);
        let remote_management =
            <super::RemoteManagementRouteKind as ControlResourceKind>::mint_handle(
                sid, lane, scope,
            );
        let remote_telemetry =
            <super::RemoteTelemetryRouteKind as ControlResourceKind>::mint_handle(sid, lane, scope);
        let remote_reject =
            <super::RemoteRejectRouteKind as ControlResourceKind>::mint_handle(sid, lane, scope);

        assert_eq!(remote_sensor, (0, scope.raw()));
        assert_eq!(remote_actuator, (1, scope.raw()));
        assert_eq!(remote_management, (2, scope.raw()));
        assert_eq!(remote_telemetry, (3, scope.raw()));
        assert_eq!(remote_reject, (4, scope.raw()));

        let network_datagram_send =
            <super::NetworkDatagramSendRouteKind as ControlResourceKind>::mint_handle(
                sid, lane, scope,
            );
        let network_datagram_recv =
            <super::NetworkDatagramRecvRouteKind as ControlResourceKind>::mint_handle(
                sid, lane, scope,
            );
        let network_stream_write =
            <super::NetworkStreamWriteRouteKind as ControlResourceKind>::mint_handle(
                sid, lane, scope,
            );
        let network_stream_read =
            <super::NetworkStreamReadRouteKind as ControlResourceKind>::mint_handle(
                sid, lane, scope,
            );
        let network_reject =
            <super::NetworkRejectRouteKind as ControlResourceKind>::mint_handle(sid, lane, scope);
        let network_accept =
            <super::NetworkAcceptRouteKind as ControlResourceKind>::mint_handle(sid, lane, scope);

        assert_eq!(network_datagram_send, (0, scope.raw()));
        assert_eq!(network_datagram_recv, (1, scope.raw()));
        assert_eq!(network_stream_write, (2, scope.raw()));
        assert_eq!(network_stream_read, (3, scope.raw()));
        assert_eq!(network_reject, (4, scope.raw()));
        assert_eq!(network_accept, (5, scope.raw()));
    }

    #[test]
    fn engine_req_round_trips_log_u32() {
        let req = EngineReq::LogU32(0x4849_4241);
        let mut buf = [0u8; 5];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_yield() {
        let req = EngineReq::Yield;
        let mut buf = [0u8; 1];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_wasip1_stdout() {
        let req = EngineReq::Wasip1Stdout(
            StdoutChunk::new_with_lease(1, b"hibana wasip1 stdout\n").expect("chunk"),
        );
        let mut buf = [0u8; 33];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_wasip1_stderr() {
        let req = EngineReq::Wasip1Stderr(
            StderrChunk::new_with_lease(2, b"hibana wasip1 stderr\n").expect("chunk"),
        );
        let mut buf = [0u8; 33];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_wasip1_stdin() {
        let req = EngineReq::Wasip1Stdin(StdinRequest::new_with_lease(3, 24).expect("request"));
        let mut buf = [0u8; 3];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_wasip1_clock_now() {
        let req = EngineReq::Wasip1ClockNow;
        let mut buf = [0u8; 1];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_wasip1_random_seed() {
        let req = EngineReq::Wasip1RandomSeed;
        let mut buf = [0u8; 1];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_wasip1_exit() {
        let req = EngineReq::Wasip1Exit(Wasip1ExitStatus::new(7));
        let mut buf = [0u8; 2];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_wasi_p1_subset() {
        let requests = [
            EngineReq::FdWrite(FdWrite::new_with_lease(1, 11, b"stdout").expect("fd_write")),
            EngineReq::FdRead(FdRead::new_with_lease(0, 12, 8).expect("fd_read")),
            EngineReq::FdFdstatGet(FdRequest::new(1)),
            EngineReq::FdClose(FdRequest::new(4)),
            EngineReq::ClockResGet(ClockResGet::new(0)),
            EngineReq::ClockTimeGet(ClockTimeGet::new(0, 1000)),
            EngineReq::PollOneoff(PollOneoff::new(44)),
            EngineReq::RandomGet(RandomGet::new_with_lease(13, 8).expect("random_get")),
            EngineReq::ProcExit(ProcExitStatus::new(7)),
            EngineReq::ArgsGet(ArgsGet::new_with_lease(14, 16).expect("args_get")),
            EngineReq::EnvironGet(EnvironGet::new_with_lease(15, 16).expect("environ_get")),
        ];
        let mut buf = [0u8; 40];
        for req in requests {
            let len = encode(&req, &mut buf);
            let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
            assert_eq!(decoded, req);
        }
    }

    #[test]
    fn engine_req_round_trips_timer_sleep_until() {
        let req = EngineReq::TimerSleepUntil(TimerSleepUntil::new(42));
        let mut buf = [0u8; 9];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_req_round_trips_gpio_set() {
        let req = EngineReq::GpioSet(GpioSet::new(25, true));
        let mut buf = [0u8; 3];
        let len = encode(&req, &mut buf);
        let decoded = EngineReq::decode_payload(Payload::new(&buf[..len])).expect("decode req");
        assert_eq!(decoded, req);
    }

    #[test]
    fn engine_ret_round_trips_logged() {
        let ret = EngineRet::Logged(0x4849_4241);
        let mut buf = [0u8; 5];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_yielded() {
        let ret = EngineRet::Yielded;
        let mut buf = [0u8; 1];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_wasip1_stdout_written() {
        let ret = EngineRet::Wasip1StdoutWritten(21);
        let mut buf = [0u8; 2];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_wasip1_stderr_written() {
        let ret = EngineRet::Wasip1StderrWritten(21);
        let mut buf = [0u8; 2];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_wasip1_stdin_read() {
        let ret = EngineRet::Wasip1StdinRead(
            StdinChunk::new_with_lease(4, b"hibana stdin\n").expect("chunk"),
        );
        let mut buf = [0u8; 33];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_wasip1_clock_now() {
        let ret = EngineRet::Wasip1ClockNow(ClockNow::new(123_456_789));
        let mut buf = [0u8; 9];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_wasip1_random_seed() {
        let ret = EngineRet::Wasip1RandomSeed(RandomSeed::new(
            0x4849_4241_5241_4e44,
            0x5345_4544_0000_0001,
        ));
        let mut buf = [0u8; 17];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_wasi_p1_subset() {
        let replies = [
            EngineRet::FdWriteDone(FdWriteDone::new(1, 6)),
            EngineRet::FdReadDone(FdReadDone::new_with_lease(0, 12, b"stdin").expect("fd_read")),
            EngineRet::FdStat(FdStat::new(1, MemRights::Write)),
            EngineRet::FdClosed(FdClosed::new(4)),
            EngineRet::ClockResolution(ClockResolution::new(1_000_000)),
            EngineRet::ClockTime(ClockNow::new(123_456_789)),
            EngineRet::PollReady(PollReady::new(1)),
            EngineRet::RandomDone(RandomDone::new_with_lease(13, b"12345678").expect("random")),
            EngineRet::ArgsDone(ArgsDone::new_with_lease(14, b"arg").expect("args")),
            EngineRet::EnvironDone(EnvironDone::new_with_lease(15, b"K=V").expect("env")),
        ];
        let mut buf = [0u8; 40];
        for ret in replies {
            let len = encode(&ret, &mut buf);
            let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
            assert_eq!(decoded, ret);
        }
    }

    #[test]
    fn engine_ret_round_trips_timer_sleep_done() {
        let ret = EngineRet::TimerSleepDone(TimerSleepDone::new(42));
        let mut buf = [0u8; 9];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn engine_ret_round_trips_gpio_set_done() {
        let ret = EngineRet::GpioSetDone(GpioSet::new(25, true));
        let mut buf = [0u8; 3];
        let len = encode(&ret, &mut buf);
        let decoded = EngineRet::decode_payload(Payload::new(&buf[..len])).expect("decode ret");
        assert_eq!(decoded, ret);
    }

    #[test]
    fn uart_payloads_round_trip() {
        let write = UartWrite::new(b"hibana uart role").expect("uart write");
        let mut write_buf = [0u8; 80];
        let write_len = encode(&write, &mut write_buf);
        assert_eq!(
            UartWrite::decode_payload(Payload::new(&write_buf[..write_len]))
                .expect("decode uart write"),
            write
        );

        let done = UartWriteDone::new(write.len() as u8);
        let mut done_buf = [0u8; 1];
        let done_len = encode(&done, &mut done_buf);
        assert_eq!(
            UartWriteDone::decode_payload(Payload::new(&done_buf[..done_len]))
                .expect("decode uart done"),
            done
        );

        assert!(UartWrite::new(&[0u8; super::UART_WRITE_CHUNK_CAPACITY + 1]).is_err());
    }

    #[test]
    fn gpio_wait_payloads_round_trip() {
        let wait = GpioWait::new(60, 7, 4, 2);
        let mut wait_buf = [0u8; 6];
        let wait_len = encode(&wait, &mut wait_buf);
        assert_eq!(
            GpioWait::decode_payload(Payload::new(&wait_buf[..wait_len])).expect("decode wait"),
            wait
        );

        let edge = GpioEdge::new(wait.wait_id(), wait.pin(), true, wait.generation());
        let mut edge_buf = [0u8; 6];
        let edge_len = encode(&edge, &mut edge_buf);
        assert_eq!(
            GpioEdge::decode_payload(Payload::new(&edge_buf[..edge_len])).expect("decode edge"),
            edge
        );
    }

    #[test]
    fn budget_control_payloads_round_trip() {
        let run = BudgetRun::new(7, 3, 1000, 123_456);
        let mut run_buf = [0u8; 16];
        let run_len = encode(&run, &mut run_buf);
        assert_eq!(
            BudgetRun::decode_payload(Payload::new(&run_buf[..run_len])).expect("decode run"),
            run
        );

        let expired = BudgetExpired::new(7, 3);
        let mut expired_buf = [0u8; 4];
        let expired_len = encode(&expired, &mut expired_buf);
        assert_eq!(
            BudgetExpired::decode_payload(Payload::new(&expired_buf[..expired_len]))
                .expect("decode expired"),
            expired
        );

        let suspend = BudgetSuspend::new(7, 3);
        let mut suspend_buf = [0u8; 4];
        let suspend_len = encode(&suspend, &mut suspend_buf);
        assert_eq!(
            BudgetSuspend::decode_payload(Payload::new(&suspend_buf[..suspend_len]))
                .expect("decode suspend"),
            suspend
        );

        let restart = BudgetRestart::new(8, 4, 500, 124_000);
        let mut restart_buf = [0u8; 16];
        let restart_len = encode(&restart, &mut restart_buf);
        assert_eq!(
            BudgetRestart::decode_payload(Payload::new(&restart_buf[..restart_len]))
                .expect("decode restart"),
            restart
        );
    }

    #[test]
    fn memory_control_payloads_round_trip() {
        let borrow = MemBorrow::new(0x1000, 21, 3);
        let mut borrow_buf = [0u8; 9];
        let borrow_len = encode(&borrow, &mut borrow_buf);
        assert_eq!(
            MemBorrow::decode_payload(Payload::new(&borrow_buf[..borrow_len]))
                .expect("decode borrow"),
            borrow
        );

        let grant = MemGrant::new(5, 0x1000, 21, 3, MemRights::Read);
        let mut grant_buf = [0u8; 11];
        let grant_len = encode(&grant, &mut grant_buf);
        assert_eq!(
            MemGrant::decode_payload(Payload::new(&grant_buf[..grant_len])).expect("decode grant"),
            grant
        );

        let release = MemRelease::new(5);
        let mut release_buf = [0u8; 1];
        let release_len = encode(&release, &mut release_buf);
        assert_eq!(
            MemRelease::decode_payload(Payload::new(&release_buf[..release_len]))
                .expect("decode release"),
            release
        );

        let commit = MemCommit::new(5, 12);
        let mut commit_buf = [0u8; 2];
        let commit_len = encode(&commit, &mut commit_buf);
        assert_eq!(
            MemCommit::decode_payload(Payload::new(&commit_buf[..commit_len]))
                .expect("decode commit"),
            commit
        );

        let fence = MemFence::new(MemFenceReason::HotSwap, 4);
        let mut fence_buf = [0u8; 5];
        let fence_len = encode(&fence, &mut fence_buf);
        assert_eq!(
            MemFence::decode_payload(Payload::new(&fence_buf[..fence_len])).expect("decode fence"),
            fence
        );
    }

    #[test]
    fn management_payloads_round_trip() {
        let begin = MgmtImageBegin::new(1, 128, 7);
        let mut begin_buf = [0u8; 9];
        let begin_len = encode(&begin, &mut begin_buf);
        assert_eq!(
            MgmtImageBegin::decode_payload(Payload::new(&begin_buf[..begin_len]))
                .expect("decode begin"),
            begin
        );

        let chunk = MgmtImageChunk::new(1, 24, b"hibana-image-chunk").expect("chunk");
        let mut chunk_buf = [0u8; 6 + MGMT_IMAGE_CHUNK_CAPACITY];
        let chunk_len = encode(&chunk, &mut chunk_buf);
        assert_eq!(
            MgmtImageChunk::decode_payload(Payload::new(&chunk_buf[..chunk_len]))
                .expect("decode chunk"),
            chunk
        );

        let end = MgmtImageEnd::new(1, 128);
        let mut end_buf = [0u8; 5];
        let end_len = encode(&end, &mut end_buf);
        assert_eq!(
            MgmtImageEnd::decode_payload(Payload::new(&end_buf[..end_len])).expect("decode end"),
            end
        );

        let activate = MgmtImageActivate::new(1, 8);
        let mut activate_buf = [0u8; 5];
        let activate_len = encode(&activate, &mut activate_buf);
        assert_eq!(
            MgmtImageActivate::decode_payload(Payload::new(&activate_buf[..activate_len]))
                .expect("decode activate"),
            activate
        );

        let rollback = MgmtImageRollback::new(0);
        let mut rollback_buf = [0u8; 1];
        let rollback_len = encode(&rollback, &mut rollback_buf);
        assert_eq!(
            MgmtImageRollback::decode_payload(Payload::new(&rollback_buf[..rollback_len]))
                .expect("decode rollback"),
            rollback
        );

        for code in [
            MgmtStatusCode::Ok,
            MgmtStatusCode::InvalidImage,
            MgmtStatusCode::NeedFence,
            MgmtStatusCode::BadSlot,
            MgmtStatusCode::RollbackEmpty,
            MgmtStatusCode::BadFenceEpoch,
            MgmtStatusCode::AuthFailed,
            MgmtStatusCode::BadSessionGeneration,
            MgmtStatusCode::ImageTooLarge,
            MgmtStatusCode::OffsetMismatch,
            MgmtStatusCode::LengthMismatch,
            MgmtStatusCode::BadChunkIndex,
        ] {
            let status = MgmtStatus::new(1, code);
            let mut status_buf = [0u8; 2];
            let status_len = encode(&status, &mut status_buf);
            assert_eq!(
                MgmtStatus::decode_payload(Payload::new(&status_buf[..status_len]))
                    .expect("decode status"),
                status
            );
        }
    }

    #[test]
    fn management_payloads_fail_closed_on_invalid_encoding() {
        let oversized = [0u8; MGMT_IMAGE_CHUNK_CAPACITY + 1];
        assert!(matches!(
            MgmtImageChunk::new(1, 0, &oversized),
            Err(CodecError::Invalid(_))
        ));
        assert!(matches!(
            MgmtStatus::decode_payload(Payload::new(&[1, 99])),
            Err(CodecError::Invalid(_))
        ));
        assert!(matches!(
            MgmtImageChunk::decode_payload(Payload::new(&[1, 0, 0, 0, 0, 4, 1])),
            Err(CodecError::Invalid(_))
        ));
    }

    #[test]
    fn timer_sleep_payloads_round_trip() {
        let request = TimerSleepUntil::new(1234);
        let mut request_buf = [0u8; 8];
        let request_len = encode(&request, &mut request_buf);
        assert_eq!(
            TimerSleepUntil::decode_payload(Payload::new(&request_buf[..request_len]))
                .expect("decode sleep request"),
            request
        );

        let reply = TimerSleepDone::new(request.tick());
        let mut reply_buf = [0u8; 8];
        let reply_len = encode(&reply, &mut reply_buf);
        assert_eq!(
            TimerSleepDone::decode_payload(Payload::new(&reply_buf[..reply_len]))
                .expect("decode sleep reply"),
            reply
        );
    }

    #[test]
    fn gpio_payload_fails_closed_on_invalid_level() {
        assert_eq!(
            GpioSet::decode_payload(Payload::new(&[25, 2])),
            Err(CodecError::Invalid("gpio level must be 0 or 1"))
        );
    }
}
