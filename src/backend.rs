use core::{cell::UnsafeCell, mem::MaybeUninit, task::Waker};

#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::arch::asm;
#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::ptr::{read_volatile, write_volatile};

use hibana::substrate::transport::TransportError;

pub(crate) const ROLE_CAPACITY: usize = 2;
pub(crate) const QUEUE_CAPACITY: usize = 8;
pub(crate) const PAYLOAD_CAPACITY: usize = 32;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_BASE: usize = 0xD000_0000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_CPUID: *const u32 = SIO_BASE as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_FIFO_ST: *const u32 = (SIO_BASE + 0x50) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_FIFO_WR: *mut u32 = (SIO_BASE + 0x54) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_FIFO_RD: *const u32 = (SIO_BASE + 0x58) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const FIFO_VLD: u32 = 1 << 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const FIFO_RDY: u32 = 1 << 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendError {
    PayloadTooLarge,
    QueueFull,
    QueueEmpty,
    RoleOutOfRange,
}

impl From<BackendError> for TransportError {
    fn from(_value: BackendError) -> Self {
        TransportError::Failed
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FrameOwned {
    label: u8,
    len: usize,
    payload: [u8; PAYLOAD_CAPACITY],
}

impl FrameOwned {
    pub(crate) fn from_bytes(label: u8, bytes: &[u8]) -> Result<Self, BackendError> {
        if bytes.len() > PAYLOAD_CAPACITY {
            return Err(BackendError::PayloadTooLarge);
        }
        let mut payload = [0u8; PAYLOAD_CAPACITY];
        payload[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            label,
            len: bytes.len(),
            payload,
        })
    }

    pub(crate) const fn label(&self) -> u8 {
        self.label
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.payload[..self.len]
    }
}

#[derive(Clone, Copy)]
struct FixedQueue {
    items: [Option<FrameOwned>; QUEUE_CAPACITY],
    head: usize,
    len: usize,
}

impl FixedQueue {
    const fn new() -> Self {
        Self {
            items: [None; QUEUE_CAPACITY],
            head: 0,
            len: 0,
        }
    }

    fn push_back(&mut self, item: FrameOwned) -> Result<(), BackendError> {
        if self.len >= QUEUE_CAPACITY {
            return Err(BackendError::QueueFull);
        }
        let idx = (self.head + self.len) % QUEUE_CAPACITY;
        self.items[idx] = Some(item);
        self.len += 1;
        Ok(())
    }

    fn push_front(&mut self, item: FrameOwned) -> Result<(), BackendError> {
        if self.len >= QUEUE_CAPACITY {
            return Err(BackendError::QueueFull);
        }
        self.head = if self.head == 0 {
            QUEUE_CAPACITY - 1
        } else {
            self.head - 1
        };
        self.items[self.head] = Some(item);
        self.len += 1;
        Ok(())
    }

    fn pop_front(&mut self) -> Option<FrameOwned> {
        if self.len == 0 {
            return None;
        }
        let idx = self.head;
        self.head = (self.head + 1) % QUEUE_CAPACITY;
        self.len -= 1;
        self.items[idx].take()
    }

    fn peek_front(&self) -> Option<&FrameOwned> {
        if self.len == 0 {
            return None;
        }
        self.items[self.head].as_ref()
    }
}

#[derive(Clone, Copy)]
struct RoleState {
    queue: FixedQueue,
}

impl RoleState {
    const fn new() -> Self {
        Self {
            queue: FixedQueue::new(),
        }
    }
}

#[derive(Clone, Copy)]
struct BackendState {
    roles: [RoleState; ROLE_CAPACITY],
}

impl BackendState {
    const fn new() -> Self {
        Self {
            roles: [RoleState::new(); ROLE_CAPACITY],
        }
    }

    fn role_mut(&mut self, role: u8) -> Result<&mut RoleState, BackendError> {
        self.roles
            .get_mut(role as usize)
            .ok_or(BackendError::RoleOutOfRange)
    }

    fn push_back(&mut self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        self.role_mut(role)?.queue.push_back(frame)
    }

    fn push_front(&mut self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        self.role_mut(role)?.queue.push_front(frame)
    }

    fn pop_front(&mut self, role: u8) -> Result<Option<FrameOwned>, BackendError> {
        Ok(self.role_mut(role)?.queue.pop_front())
    }

    fn peek_label(&self, role: u8) -> Result<Option<u8>, BackendError> {
        self.roles
            .get(role as usize)
            .map(|state| state.queue.peek_front().map(FrameOwned::label))
            .ok_or(BackendError::RoleOutOfRange)
    }

    #[cfg(test)]
    fn queue_len(&self, role: u8) -> Result<usize, BackendError> {
        self.roles
            .get(role as usize)
            .map(|state| state.queue.len)
            .ok_or(BackendError::RoleOutOfRange)
    }
}

struct StoredWaker {
    present: bool,
    waker: MaybeUninit<Waker>,
}

impl StoredWaker {
    const fn new() -> Self {
        Self {
            present: false,
            waker: MaybeUninit::uninit(),
        }
    }

    fn store(&mut self, waker: &Waker) {
        if self.present {
            unsafe {
                self.waker.as_mut_ptr().replace(waker.clone());
            }
        } else {
            self.waker.write(waker.clone());
            self.present = true;
        }
    }

    fn take(&mut self) -> Option<Waker> {
        if !self.present {
            return None;
        }
        self.present = false;
        Some(unsafe { self.waker.assume_init_read() })
    }
}

struct WakerState {
    roles: [StoredWaker; ROLE_CAPACITY],
}

impl WakerState {
    const fn new() -> Self {
        Self {
            roles: [const { StoredWaker::new() }; ROLE_CAPACITY],
        }
    }

    fn store(&mut self, role: u8, waker: &Waker) -> Result<(), BackendError> {
        self.roles
            .get_mut(role as usize)
            .ok_or(BackendError::RoleOutOfRange)?
            .store(waker);
        Ok(())
    }

    fn take(&mut self, role: u8) -> Result<Option<Waker>, BackendError> {
        Ok(self
            .roles
            .get_mut(role as usize)
            .ok_or(BackendError::RoleOutOfRange)?
            .take())
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
struct RequeueSlots(UnsafeCell<[Option<FrameOwned>; ROLE_CAPACITY]>);

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl Sync for RequeueSlots {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
static RP2040_REQUEUE_SLOTS: RequeueSlots = RequeueSlots(UnsafeCell::new([None; ROLE_CAPACITY]));

#[cfg(all(target_arch = "arm", target_os = "none"))]
struct Rp2040RecvWakers(UnsafeCell<WakerState>);

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl Sync for Rp2040RecvWakers {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
static RP2040_RECV_WAKERS: Rp2040RecvWakers = Rp2040RecvWakers(UnsafeCell::new(WakerState::new()));

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_validate_role(role: u8) -> Result<usize, BackendError> {
    if (role as usize) < ROLE_CAPACITY {
        Ok(role as usize)
    } else {
        Err(BackendError::RoleOutOfRange)
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_fifo_word_count(len: usize) -> usize {
    len.div_ceil(4)
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_core_id() -> u8 {
    unsafe { read_volatile(SIO_CPUID) as u8 }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_fifo_status() -> u32 {
    unsafe { read_volatile(SIO_FIFO_ST) }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_sev() {
    unsafe { asm!("sev", options(nomem, nostack, preserves_flags)) }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_wfe() {
    unsafe { asm!("wfe", options(nomem, nostack, preserves_flags)) }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_fifo_try_read() -> Option<u32> {
    if rp2040_fifo_status() & FIFO_VLD == 0 {
        return None;
    }
    Some(unsafe { read_volatile(SIO_FIFO_RD) })
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_fifo_read_blocking() -> u32 {
    loop {
        if let Some(word) = rp2040_fifo_try_read() {
            return word;
        }
        rp2040_wfe();
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_fifo_write_blocking(word: u32) {
    while rp2040_fifo_status() & FIFO_RDY == 0 {
        rp2040_wfe();
    }
    unsafe { write_volatile(SIO_FIFO_WR, word) };
    rp2040_sev();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_pack_header(frame: FrameOwned) -> u32 {
    (frame.label as u32) | (((frame.len as u32) & 0xff) << 8)
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_pack_payload_word(bytes: &[u8]) -> u32 {
    let mut word_bytes = [0u8; 4];
    word_bytes[..bytes.len()].copy_from_slice(bytes);
    u32::from_le_bytes(word_bytes)
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_requeue_take(role: u8) -> Result<Option<FrameOwned>, BackendError> {
    let slot = rp2040_validate_role(role)?;
    let slots = unsafe { &mut *RP2040_REQUEUE_SLOTS.0.get() };
    Ok(slots[slot].take())
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_store_recv_waker(role: u8, waker: &Waker) -> Result<(), BackendError> {
    let recv_wakers = unsafe { &mut *RP2040_RECV_WAKERS.0.get() };
    recv_wakers.store(role, waker)
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_take_recv_waker(role: u8) -> Result<Option<Waker>, BackendError> {
    let recv_wakers = unsafe { &mut *RP2040_RECV_WAKERS.0.get() };
    recv_wakers.take(role)
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_wake_recv(role: u8) -> Result<(), BackendError> {
    if let Some(waker) = rp2040_take_recv_waker(role)? {
        waker.wake();
    }
    Ok(())
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_requeue_put(role: u8, frame: FrameOwned) -> Result<(), BackendError> {
    let slot = rp2040_validate_role(role)?;
    let slots = unsafe { &mut *RP2040_REQUEUE_SLOTS.0.get() };
    if slots[slot].is_some() {
        return Err(BackendError::QueueFull);
    }
    slots[slot] = Some(frame);
    let _ = rp2040_wake_recv(role);
    Ok(())
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_enqueue(role: u8, frame: FrameOwned) -> Result<(), BackendError> {
    let _ = rp2040_validate_role(role)?;
    if frame.len > PAYLOAD_CAPACITY {
        return Err(BackendError::PayloadTooLarge);
    }
    rp2040_fifo_write_blocking(rp2040_pack_header(frame));
    for chunk in frame.as_slice().chunks(4) {
        rp2040_fifo_write_blocking(rp2040_pack_payload_word(chunk));
    }
    let _ = rp2040_wake_recv(role);
    Ok(())
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_dequeue(role: u8) -> Result<Option<FrameOwned>, BackendError> {
    let _ = rp2040_validate_role(role)?;
    if let Some(frame) = rp2040_requeue_take(role)? {
        let _ = rp2040_take_recv_waker(role)?;
        return Ok(Some(frame));
    }

    let Some(header) = rp2040_fifo_try_read() else {
        return Ok(None);
    };

    let label = (header & 0xff) as u8;
    let len = ((header >> 8) & 0xff) as usize;
    let mut payload = [0u8; PAYLOAD_CAPACITY];
    let word_count = rp2040_fifo_word_count(len);

    if len > PAYLOAD_CAPACITY {
        for _ in 0..word_count {
            let _ = rp2040_fifo_read_blocking();
        }
        return Err(BackendError::PayloadTooLarge);
    }

    for word_index in 0..word_count {
        let word = rp2040_fifo_read_blocking();
        let start = word_index * 4;
        let end = core::cmp::min(start + 4, len);
        payload[start..end].copy_from_slice(&word.to_le_bytes()[..(end - start)]);
    }

    let frame = FrameOwned {
        label,
        len,
        payload,
    };
    let _ = rp2040_take_recv_waker(role)?;
    Ok(Some(frame))
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn rp2040_peek_label(role: u8) -> Result<Option<u8>, BackendError> {
    let _ = rp2040_validate_role(role)?;
    let slots = unsafe { &*RP2040_REQUEUE_SLOTS.0.get() };
    Ok(slots[role as usize].map(|frame| frame.label()))
}

pub(crate) trait FifoBackend {
    fn enqueue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError>;
    fn dequeue(&self, role: u8) -> Result<Option<FrameOwned>, BackendError>;
    fn requeue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError>;
    fn peek_label(&self, role: u8) -> Result<Option<u8>, BackendError>;
    fn store_recv_waker(&self, role: u8, waker: &Waker) -> Result<(), BackendError>;
}

/// Host-side fixed-capacity queue backend used by parity tests.
pub struct HostQueueBackend {
    state: UnsafeCell<BackendState>,
    recv_wakers: UnsafeCell<WakerState>,
}

impl HostQueueBackend {
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(BackendState::new()),
            recv_wakers: UnsafeCell::new(WakerState::new()),
        }
    }

    fn with_state_mut<R>(&self, f: impl FnOnce(&mut BackendState) -> R) -> R {
        unsafe { f(&mut *self.state.get()) }
    }

    fn with_recv_wakers_mut<R>(&self, f: impl FnOnce(&mut WakerState) -> R) -> R {
        unsafe { f(&mut *self.recv_wakers.get()) }
    }

    #[cfg(test)]
    pub(crate) fn enqueue_bytes(&self, role: u8, bytes: &[u8]) -> Result<(), BackendError> {
        self.enqueue(role, FrameOwned::from_bytes(0, bytes)?)
    }

    #[cfg(test)]
    pub(crate) fn queue_len(&self, role: u8) -> Result<usize, BackendError> {
        self.with_state_mut(|state| state.queue_len(role))
    }
}

impl Default for HostQueueBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FifoBackend for HostQueueBackend {
    fn enqueue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        self.with_state_mut(|state| state.push_back(role, frame))?;
        if let Some(waker) = self.with_recv_wakers_mut(|state| state.take(role))? {
            waker.wake();
        }
        Ok(())
    }

    fn dequeue(&self, role: u8) -> Result<Option<FrameOwned>, BackendError> {
        let frame = self.with_state_mut(|state| state.pop_front(role))?;
        if frame.is_some() {
            let _ = self.with_recv_wakers_mut(|state| state.take(role))?;
        }
        Ok(frame)
    }

    fn requeue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        self.with_state_mut(|state| state.push_front(role, frame))?;
        if let Some(waker) = self.with_recv_wakers_mut(|state| state.take(role))? {
            waker.wake();
        }
        Ok(())
    }

    fn peek_label(&self, role: u8) -> Result<Option<u8>, BackendError> {
        self.with_state_mut(|state| state.peek_label(role))
    }

    fn store_recv_waker(&self, role: u8, waker: &Waker) -> Result<(), BackendError> {
        self.with_recv_wakers_mut(|state| state.store(role, waker))
    }
}

pub struct Rp2040SioBackend {
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    state: UnsafeCell<BackendState>,
}

impl Rp2040SioBackend {
    pub const fn new() -> Self {
        Self {
            #[cfg(not(all(target_arch = "arm", target_os = "none")))]
            state: UnsafeCell::new(BackendState::new()),
        }
    }

    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    fn with_state_mut<R>(&self, f: impl FnOnce(&mut BackendState) -> R) -> R {
        unsafe { f(&mut *self.state.get()) }
    }
}

impl Default for Rp2040SioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FifoBackend for Rp2040SioBackend {
    fn enqueue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            let _ = rp2040_core_id();
            return rp2040_enqueue(role, frame);
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        self.with_state_mut(|state| state.push_back(role, frame))
    }

    fn dequeue(&self, role: u8) -> Result<Option<FrameOwned>, BackendError> {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            let _ = rp2040_core_id();
            return rp2040_dequeue(role);
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        self.with_state_mut(|state| state.pop_front(role))
    }

    fn requeue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            return rp2040_requeue_put(role, frame);
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        self.with_state_mut(|state| state.push_front(role, frame))
    }

    fn peek_label(&self, role: u8) -> Result<Option<u8>, BackendError> {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            return rp2040_peek_label(role);
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        self.with_state_mut(|state| state.peek_label(role))
    }

    fn store_recv_waker(&self, role: u8, waker: &Waker) -> Result<(), BackendError> {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            return rp2040_store_recv_waker(role, waker);
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        {
            let _ = (role, waker);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendError, HostQueueBackend, PAYLOAD_CAPACITY, QUEUE_CAPACITY};

    #[test]
    fn host_backend_rejects_payloads_that_exceed_fixed_capacity() {
        let backend = HostQueueBackend::new();
        let payload = [7u8; PAYLOAD_CAPACITY + 1];
        let result = backend.enqueue_bytes(0, &payload);
        assert_eq!(result, Err(BackendError::PayloadTooLarge));
    }

    #[test]
    fn host_backend_rejects_queue_overflow() {
        let backend = HostQueueBackend::new();
        for _ in 0..QUEUE_CAPACITY {
            backend.enqueue_bytes(0, &[1, 2, 3]).expect("room in queue");
        }
        assert_eq!(backend.enqueue_bytes(0, &[9]), Err(BackendError::QueueFull));
        assert_eq!(backend.queue_len(0), Ok(QUEUE_CAPACITY));
    }
}
