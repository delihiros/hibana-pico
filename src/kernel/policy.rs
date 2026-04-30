use hibana::{
    g::Msg,
    substrate::wire::{CodecError, Payload, WireEncode, WirePayload},
};

use crate::{
    choreography::protocol::{
        LABEL_SWARM_JOIN_ACK, LABEL_SWARM_JOIN_GRANT, LABEL_SWARM_JOIN_OFFER,
        LABEL_SWARM_JOIN_REQUEST, LABEL_SWARM_LEAVE_ACK, LABEL_SWARM_NODE_IMAGE_UPDATED,
        LABEL_SWARM_NODE_REVOKED, LABEL_SWARM_POLICY_APP0, LABEL_SWARM_POLICY_APP1,
        LABEL_SWARM_REVOKE_REMOTE_OBJECTS, LABEL_SWARM_SUSPEND, LABEL_SWARM_TELEMETRY,
    },
    kernel::swarm::NodeId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyError {
    BadApp,
    BadSlot,
    Disabled,
    TelemetryBlocked,
    TableFull,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicySlotState {
    slot: u8,
    enabled: bool,
}

impl PolicySlotState {
    pub const fn enabled(slot: u8) -> Self {
        Self {
            slot,
            enabled: true,
        }
    }

    pub const fn disabled(slot: u8) -> Self {
        Self {
            slot,
            enabled: false,
        }
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicySlotTable<const N: usize> {
    slots: [Option<PolicySlotState>; N],
}

impl<const N: usize> PolicySlotTable<N> {
    pub const fn new() -> Self {
        Self { slots: [None; N] }
    }

    pub fn allow(&mut self, slot: u8) -> Result<(), PolicyError> {
        self.upsert(PolicySlotState::enabled(slot))
    }

    pub fn deny(&mut self, slot: u8) -> Result<(), PolicyError> {
        self.upsert(PolicySlotState::disabled(slot))
    }

    pub fn validate(&self, slot: u8) -> Result<(), PolicyError> {
        let state = self
            .slots
            .iter()
            .flatten()
            .find(|state| state.slot == slot)
            .copied()
            .ok_or(PolicyError::BadSlot)?;
        if !state.enabled {
            return Err(PolicyError::Disabled);
        }
        Ok(())
    }

    pub fn is_allowed(&self, slot: u8) -> bool {
        self.validate(slot).is_ok()
    }

    fn upsert(&mut self, state: PolicySlotState) -> Result<(), PolicyError> {
        if let Some(existing) = self
            .slots
            .iter_mut()
            .find(|existing| existing.is_some_and(|existing| existing.slot == state.slot))
        {
            *existing = Some(state);
            return Ok(());
        }
        let slot = self
            .slots
            .iter_mut()
            .find(|slot| slot.is_none())
            .ok_or(PolicyError::TableFull)?;
        *slot = Some(state);
        Ok(())
    }
}

impl<const N: usize> Default for PolicySlotTable<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeRole {
    Coordinator,
    Sensor,
    Actuator,
    Gateway,
}

impl NodeRole {
    pub const fn bit(self) -> u16 {
        match self {
            Self::Coordinator => 0b0001,
            Self::Sensor => 0b0010,
            Self::Actuator => 0b0100,
            Self::Gateway => 0b1000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleMask(u16);

impl RoleMask {
    pub const fn new(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn single(role: NodeRole) -> Self {
        Self(role.bit())
    }

    pub const fn with(self, role: NodeRole) -> Self {
        Self(self.0 | role.bit())
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, role: NodeRole) -> bool {
        self.0 & role.bit() != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwarmTelemetry {
    node_id: NodeId,
    role_mask: RoleMask,
    queue_depth: u8,
    link_loss: u8,
    budget_remaining: u16,
    temperature_milli_c: i16,
    session_generation: u16,
}

impl SwarmTelemetry {
    pub const fn new(
        node_id: NodeId,
        role_mask: RoleMask,
        queue_depth: u8,
        link_loss: u8,
        budget_remaining: u16,
        temperature_milli_c: i16,
        session_generation: u16,
    ) -> Self {
        Self {
            node_id,
            role_mask,
            queue_depth,
            link_loss,
            budget_remaining,
            temperature_milli_c,
            session_generation,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn role_mask(&self) -> RoleMask {
        self.role_mask
    }

    pub const fn queue_depth(&self) -> u8 {
        self.queue_depth
    }

    pub const fn link_loss(&self) -> u8 {
        self.link_loss
    }

    pub const fn budget_remaining(&self) -> u16 {
        self.budget_remaining
    }

    pub const fn temperature_milli_c(&self) -> i16 {
        self.temperature_milli_c
    }

    pub const fn session_generation(&self) -> u16 {
        self.session_generation
    }

    pub const fn blocks_runtime_authority(&self) -> bool {
        self.queue_depth > 16
            || self.link_loss > 32
            || self.budget_remaining == 0
            || self.temperature_milli_c > 85_00
    }
}

impl WireEncode for SwarmTelemetry {
    fn encoded_len(&self) -> Option<usize> {
        Some(12)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 12 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.role_mask.bits().to_be_bytes());
        out[4] = self.queue_depth;
        out[5] = self.link_loss;
        out[6..8].copy_from_slice(&self.budget_remaining.to_be_bytes());
        out[8..10].copy_from_slice(&self.temperature_milli_c.to_be_bytes());
        out[10..12].copy_from_slice(&self.session_generation.to_be_bytes());
        Ok(12)
    }
}

impl WirePayload for SwarmTelemetry {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 12 {
            return Err(CodecError::Invalid("swarm telemetry carries twelve bytes"));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            RoleMask::new(u16::from_be_bytes([bytes[2], bytes[3]])),
            bytes[4],
            bytes[5],
            u16::from_be_bytes([bytes[6], bytes[7]]),
            i16::from_be_bytes([bytes[8], bytes[9]]),
            u16::from_be_bytes([bytes[10], bytes[11]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeImageUpdated {
    node_id: NodeId,
    slot: u8,
    image_generation: u32,
    accepted: bool,
}

impl NodeImageUpdated {
    pub const fn new(node_id: NodeId, slot: u8, image_generation: u32, accepted: bool) -> Self {
        Self {
            node_id,
            slot,
            image_generation,
            accepted,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn slot(&self) -> u8 {
        self.slot
    }

    pub const fn image_generation(&self) -> u32 {
        self.image_generation
    }

    pub const fn accepted(&self) -> bool {
        self.accepted
    }
}

impl WireEncode for NodeImageUpdated {
    fn encoded_len(&self) -> Option<usize> {
        Some(8)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 8 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2] = self.slot;
        out[3..7].copy_from_slice(&self.image_generation.to_be_bytes());
        out[7] = u8::from(self.accepted);
        Ok(8)
    }
}

impl WirePayload for NodeImageUpdated {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 8 {
            return Err(CodecError::Invalid("node image update carries eight bytes"));
        }
        let accepted = match bytes[7] {
            0 => false,
            1 => true,
            _ => return Err(CodecError::Invalid("node image update boolean")),
        };
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            bytes[2],
            u32::from_be_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]),
            accepted,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeRevoked {
    node_id: NodeId,
    session_generation: u16,
}

impl NodeRevoked {
    pub const fn new(node_id: NodeId, session_generation: u16) -> Self {
        Self {
            node_id,
            session_generation,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn session_generation(&self) -> u16 {
        self.session_generation
    }
}

impl WireEncode for NodeRevoked {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.session_generation.to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for NodeRevoked {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 4 {
            return Err(CodecError::Invalid("node revoke carries four bytes"));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwarmSuspend {
    node_id: NodeId,
    session_generation: u16,
}

impl SwarmSuspend {
    pub const fn new(node_id: NodeId, session_generation: u16) -> Self {
        Self {
            node_id,
            session_generation,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn session_generation(&self) -> u16 {
        self.session_generation
    }
}

impl WireEncode for SwarmSuspend {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.session_generation.to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for SwarmSuspend {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 4 {
            return Err(CodecError::Invalid("swarm suspend carries four bytes"));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteObjectsRevoke {
    node_id: NodeId,
    session_generation: u16,
}

impl RemoteObjectsRevoke {
    pub const fn new(node_id: NodeId, session_generation: u16) -> Self {
        Self {
            node_id,
            session_generation,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn session_generation(&self) -> u16 {
        self.session_generation
    }
}

impl WireEncode for RemoteObjectsRevoke {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.session_generation.to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for RemoteObjectsRevoke {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 4 {
            return Err(CodecError::Invalid(
                "remote object revoke carries four bytes",
            ));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaveAck {
    node_id: NodeId,
    session_generation: u16,
}

impl LeaveAck {
    pub const fn new(node_id: NodeId, session_generation: u16) -> Self {
        Self {
            node_id,
            session_generation,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn session_generation(&self) -> u16 {
        self.session_generation
    }
}

impl WireEncode for LeaveAck {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.session_generation.to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for LeaveAck {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 4 {
            return Err(CodecError::Invalid("leave ack carries four bytes"));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinOffer {
    node_id: NodeId,
    role_mask: RoleMask,
}

impl JoinOffer {
    pub const fn new(node_id: NodeId, role_mask: RoleMask) -> Self {
        Self { node_id, role_mask }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn role_mask(&self) -> RoleMask {
        self.role_mask
    }
}

impl WireEncode for JoinOffer {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.role_mask.bits().to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for JoinOffer {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 4 {
            return Err(CodecError::Invalid("join offer carries four bytes"));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            RoleMask::new(u16::from_be_bytes([bytes[2], bytes[3]])),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinRequest {
    node_id: NodeId,
    role_mask: RoleMask,
}

impl JoinRequest {
    pub const fn new(node_id: NodeId, role_mask: RoleMask) -> Self {
        Self { node_id, role_mask }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn role_mask(&self) -> RoleMask {
        self.role_mask
    }
}

impl WireEncode for JoinRequest {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.role_mask.bits().to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for JoinRequest {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 4 {
            return Err(CodecError::Invalid("join request carries four bytes"));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            RoleMask::new(u16::from_be_bytes([bytes[2], bytes[3]])),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinGrant {
    node_id: NodeId,
    role_mask: RoleMask,
    session_generation: u16,
    accepted: bool,
}

impl JoinGrant {
    pub const fn new(
        node_id: NodeId,
        role_mask: RoleMask,
        session_generation: u16,
        accepted: bool,
    ) -> Self {
        Self {
            node_id,
            role_mask,
            session_generation,
            accepted,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn role_mask(&self) -> RoleMask {
        self.role_mask
    }

    pub const fn session_generation(&self) -> u16 {
        self.session_generation
    }

    pub const fn accepted(&self) -> bool {
        self.accepted
    }
}

impl WireEncode for JoinGrant {
    fn encoded_len(&self) -> Option<usize> {
        Some(7)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 7 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.role_mask.bits().to_be_bytes());
        out[4..6].copy_from_slice(&self.session_generation.to_be_bytes());
        out[6] = u8::from(self.accepted);
        Ok(7)
    }
}

impl WirePayload for JoinGrant {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 7 {
            return Err(CodecError::Invalid("join grant carries seven bytes"));
        }
        let accepted = match bytes[6] {
            0 => false,
            1 => true,
            _ => return Err(CodecError::Invalid("join grant boolean")),
        };
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            RoleMask::new(u16::from_be_bytes([bytes[2], bytes[3]])),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            accepted,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinAck {
    node_id: NodeId,
    session_generation: u16,
}

impl JoinAck {
    pub const fn new(node_id: NodeId, session_generation: u16) -> Self {
        Self {
            node_id,
            session_generation,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn session_generation(&self) -> u16 {
        self.session_generation
    }
}

impl WireEncode for JoinAck {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2..4].copy_from_slice(&self.session_generation.to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for JoinAck {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 4 {
            return Err(CodecError::Invalid("join ack carries four bytes"));
        }
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppChoice {
    App0,
    App1,
}

impl AppChoice {
    pub const fn index(self) -> usize {
        match self {
            Self::App0 => 0,
            Self::App1 => 1,
        }
    }

    pub const fn label(self) -> u8 {
        match self {
            Self::App0 => LABEL_SWARM_POLICY_APP0,
            Self::App1 => LABEL_SWARM_POLICY_APP1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppInstance {
    choice: AppChoice,
    memory_generation: u32,
    fd_generation: u16,
    lease_generation: u16,
    enabled: bool,
}

impl AppInstance {
    pub const fn new(
        choice: AppChoice,
        memory_generation: u32,
        fd_generation: u16,
        lease_generation: u16,
    ) -> Self {
        Self {
            choice,
            memory_generation,
            fd_generation,
            lease_generation,
            enabled: true,
        }
    }

    pub const fn disabled(choice: AppChoice) -> Self {
        Self {
            choice,
            memory_generation: 0,
            fd_generation: 0,
            lease_generation: 0,
            enabled: false,
        }
    }

    pub const fn choice(&self) -> AppChoice {
        self.choice
    }

    pub const fn fd_generation(&self) -> u16 {
        self.fd_generation
    }

    pub const fn lease_generation(&self) -> u16 {
        self.lease_generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppPolicySignal {
    node_id: NodeId,
    choice: AppChoice,
    route_label: u8,
    memory_generation: u32,
    fd_generation: u16,
    lease_generation: u16,
}

impl AppPolicySignal {
    pub const fn new(
        node_id: NodeId,
        choice: AppChoice,
        route_label: u8,
        memory_generation: u32,
        fd_generation: u16,
        lease_generation: u16,
    ) -> Self {
        Self {
            node_id,
            choice,
            route_label,
            memory_generation,
            fd_generation,
            lease_generation,
        }
    }

    pub const fn choice(&self) -> AppChoice {
        self.choice
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn route_label(&self) -> u8 {
        self.route_label
    }

    pub const fn memory_generation(&self) -> u32 {
        self.memory_generation
    }

    pub const fn fd_generation(&self) -> u16 {
        self.fd_generation
    }

    pub const fn lease_generation(&self) -> u16 {
        self.lease_generation
    }
}

impl WireEncode for AppPolicySignal {
    fn encoded_len(&self) -> Option<usize> {
        Some(12)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 12 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.node_id.raw().to_be_bytes());
        out[2] = self.choice.index() as u8;
        out[3] = self.route_label;
        out[4..8].copy_from_slice(&self.memory_generation.to_be_bytes());
        out[8..10].copy_from_slice(&self.fd_generation.to_be_bytes());
        out[10..12].copy_from_slice(&self.lease_generation.to_be_bytes());
        Ok(12)
    }
}

impl WirePayload for AppPolicySignal {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() != 12 {
            return Err(CodecError::Invalid(
                "app policy signal carries twelve bytes",
            ));
        }
        let choice = match bytes[2] {
            0 => AppChoice::App0,
            1 => AppChoice::App1,
            _ => return Err(CodecError::Invalid("app policy choice")),
        };
        Ok(Self::new(
            NodeId::new(u16::from_be_bytes([bytes[0], bytes[1]])),
            choice,
            bytes[3],
            u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            u16::from_be_bytes([bytes[8], bytes[9]]),
            u16::from_be_bytes([bytes[10], bytes[11]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiAppPolicyState {
    slots: [AppInstance; 2],
}

impl MultiAppPolicyState {
    pub const fn new(app0: AppInstance, app1: AppInstance) -> Self {
        Self {
            slots: [app0, app1],
        }
    }

    pub fn choose_explicit(
        &self,
        node_id: NodeId,
        choice: AppChoice,
        route_label: u8,
        telemetry: SwarmTelemetry,
    ) -> Result<AppPolicySignal, PolicyError> {
        if telemetry.blocks_runtime_authority() {
            return Err(PolicyError::TelemetryBlocked);
        }
        let slot = self.slots.get(choice.index()).ok_or(PolicyError::BadApp)?;
        if slot.choice() != choice {
            return Err(PolicyError::BadApp);
        }
        if !slot.enabled {
            return Err(PolicyError::Disabled);
        }
        Ok(AppPolicySignal::new(
            node_id,
            choice,
            route_label,
            slot.memory_generation,
            slot.fd_generation,
            slot.lease_generation,
        ))
    }
}

pub type SwarmTelemetryMsg = Msg<LABEL_SWARM_TELEMETRY, SwarmTelemetry>;
pub type NodeImageUpdatedMsg = Msg<LABEL_SWARM_NODE_IMAGE_UPDATED, NodeImageUpdated>;
pub type NodeRevokedMsg = Msg<LABEL_SWARM_NODE_REVOKED, NodeRevoked>;
pub type SwarmSuspendMsg = Msg<LABEL_SWARM_SUSPEND, SwarmSuspend>;
pub type RemoteObjectsRevokeMsg = Msg<LABEL_SWARM_REVOKE_REMOTE_OBJECTS, RemoteObjectsRevoke>;
pub type LeaveAckMsg = Msg<LABEL_SWARM_LEAVE_ACK, LeaveAck>;
pub type JoinOfferMsg = Msg<LABEL_SWARM_JOIN_OFFER, JoinOffer>;
pub type JoinRequestMsg = Msg<LABEL_SWARM_JOIN_REQUEST, JoinRequest>;
pub type JoinGrantMsg = Msg<LABEL_SWARM_JOIN_GRANT, JoinGrant>;
pub type JoinAckMsg = Msg<LABEL_SWARM_JOIN_ACK, JoinAck>;
pub type PolicyApp0Msg = Msg<LABEL_SWARM_POLICY_APP0, AppPolicySignal>;
pub type PolicyApp1Msg = Msg<LABEL_SWARM_POLICY_APP1, AppPolicySignal>;

#[cfg(test)]
mod tests {
    use super::{PolicyError, PolicySlotTable};

    #[test]
    fn policy_slot_table_requires_explicit_allowed_slot() {
        let mut policy: PolicySlotTable<2> = PolicySlotTable::new();
        assert_eq!(policy.validate(3), Err(PolicyError::BadSlot));
        assert!(!policy.is_allowed(3));

        policy.allow(3).expect("allow policy slot");
        assert_eq!(policy.validate(3), Ok(()));
        assert!(policy.is_allowed(3));

        policy.deny(3).expect("deny policy slot");
        assert_eq!(policy.validate(3), Err(PolicyError::Disabled));
        assert!(!policy.is_allowed(3));

        policy.allow(4).expect("allow second policy slot");
        assert_eq!(policy.allow(5), Err(PolicyError::TableFull));
    }
}
