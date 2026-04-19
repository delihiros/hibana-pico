use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use hibana::substrate::{
    Transport,
    transport::{Outgoing, TransportEvent, TransportMetrics, TransportSnapshot},
    wire::Payload,
};

use crate::backend::{BackendError, FifoBackend, FrameOwned};

impl<T> FifoBackend for &T
where
    T: FifoBackend + ?Sized,
{
    fn enqueue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        (*self).enqueue(role, frame)
    }

    fn dequeue(&self, role: u8) -> Result<Option<FrameOwned>, BackendError> {
        (*self).dequeue(role)
    }

    fn requeue(&self, role: u8, frame: FrameOwned) -> Result<(), BackendError> {
        (*self).requeue(role, frame)
    }

    fn peek_label(&self, role: u8) -> Result<Option<u8>, BackendError> {
        (*self).peek_label(role)
    }

    fn store_recv_waker(&self, role: u8, waker: &Waker) -> Result<(), BackendError> {
        (*self).store_recv_waker(role, waker)
    }
}

pub struct SioTransport<B> {
    backend: B,
    snapshot: TransportSnapshot,
}

impl<B> SioTransport<B> {
    pub const fn new(backend: B) -> Self {
        Self {
            backend,
            snapshot: TransportSnapshot::new(None, None),
        }
    }
}

#[doc(hidden)]
pub struct PicoTx;

#[doc(hidden)]
pub struct PicoRx<'a, B> {
    backend: &'a B,
    role: u8,
    current: Option<FrameOwned>,
}

#[doc(hidden)]
pub struct PicoSendFuture<'a, B> {
    backend: &'a B,
    role: u8,
    frame: Option<FrameOwned>,
    error: Option<BackendError>,
}

impl<'a, B> Future for PicoSendFuture<'a, B>
where
    B: FifoBackend,
{
    type Output = Result<(), BackendError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(error) = this.error.take() {
            return Poll::Ready(Err(error));
        }
        let frame = this
            .frame
            .take()
            .expect("send future polled after completion");
        Poll::Ready(this.backend.enqueue(this.role, frame))
    }
}

#[doc(hidden)]
pub struct PicoRecvFuture<'a, B> {
    rx: &'a mut PicoRx<'a, B>,
}

impl<'a, B> Future for PicoRecvFuture<'a, B>
where
    B: FifoBackend,
{
    type Output = Result<Payload<'a>, BackendError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.rx.current.is_none() {
            this.rx.current = this.rx.backend.dequeue(this.rx.role)?;
            if this.rx.current.is_none() {
                this.rx.backend.store_recv_waker(this.rx.role, cx.waker())?;
                this.rx.current = this.rx.backend.dequeue(this.rx.role)?;
            }
        }
        let Some(frame) = this.rx.current.as_ref() else {
            return Poll::Pending;
        };
        let bytes: &'a [u8] = unsafe { &*(frame.as_slice() as *const [u8]) };
        Poll::Ready(Ok(Payload::new(bytes)))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SioTransportMetrics {
    snapshot: TransportSnapshot,
}

impl TransportMetrics for SioTransportMetrics {
    fn snapshot(&self) -> TransportSnapshot {
        self.snapshot
    }
}

impl<B> Transport for SioTransport<B>
where
    B: FifoBackend,
{
    type Error = BackendError;
    type Tx<'a>
        = PicoTx
    where
        Self: 'a;
    type Rx<'a>
        = PicoRx<'a, B>
    where
        Self: 'a;
    type Send<'a>
        = PicoSendFuture<'a, B>
    where
        Self: 'a;
    type Recv<'a>
        = PicoRecvFuture<'a, B>
    where
        Self: 'a;
    type Metrics = SioTransportMetrics;

    fn open<'a>(&'a self, local_role: u8, _session_id: u32) -> (Self::Tx<'a>, Self::Rx<'a>) {
        (
            PicoTx,
            PicoRx {
                backend: &self.backend,
                role: local_role,
                current: None,
            },
        )
    }

    fn send<'a, 'f>(&'a self, _tx: &'a mut Self::Tx<'a>, outgoing: Outgoing<'f>) -> Self::Send<'a>
    where
        'a: 'f,
    {
        let (frame, error) =
            match FrameOwned::from_bytes(outgoing.meta.label, outgoing.payload.as_bytes()) {
                Ok(frame) => (Some(frame), None),
                Err(error) => (None, Some(error)),
            };
        PicoSendFuture {
            backend: &self.backend,
            role: outgoing.meta.peer,
            frame,
            error,
        }
    }

    fn recv<'a>(&'a self, rx: &'a mut Self::Rx<'a>) -> Self::Recv<'a> {
        rx.current = None;
        PicoRecvFuture { rx }
    }

    fn requeue<'a>(&'a self, rx: &'a mut Self::Rx<'a>) {
        if let Some(frame) = rx.current.take() {
            rx.backend
                .requeue(rx.role, frame)
                .expect("requeue must preserve the previously received frame");
        }
    }

    fn drain_events(&self, _emit: &mut dyn FnMut(TransportEvent)) {}

    fn recv_label_hint<'a>(&'a self, rx: &'a Self::Rx<'a>) -> Option<u8> {
        self.backend.peek_label(rx.role).ok().flatten()
    }

    fn metrics(&self) -> Self::Metrics {
        SioTransportMetrics {
            snapshot: self.snapshot,
        }
    }

    fn apply_pacing_update(&self, _interval_us: u32, _burst_bytes: u16) {}
}

#[cfg(test)]
mod tests {
    use super::SioTransport;
    use crate::backend::{FifoBackend, FrameOwned, HostQueueBackend};
    use core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };
    use hibana::substrate::Transport;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct CountWake {
        wakes: AtomicUsize,
    }

    impl std::task::Wake for CountWake {
        fn wake(self: Arc<Self>) {
            self.wakes.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.wakes.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn recv_future_registers_waker_and_enqueue_wakes_it() {
        let backend = HostQueueBackend::new();
        let transport = SioTransport::new(&backend);
        let (_tx, mut rx) = transport.open(0, 7);
        let wake_counter = Arc::new(CountWake {
            wakes: AtomicUsize::new(0),
        });
        let waker = std::task::Waker::from(Arc::clone(&wake_counter));
        let mut cx = Context::from_waker(&waker);
        let mut recv = transport.recv(&mut rx);

        assert!(matches!(Pin::new(&mut recv).poll(&mut cx), Poll::Pending));
        assert_eq!(wake_counter.wakes.load(Ordering::SeqCst), 0);

        backend
            .enqueue(0, FrameOwned::from_bytes(1, &[0x2a]).expect("frame"))
            .expect("enqueue should wake pending recv");

        assert_eq!(wake_counter.wakes.load(Ordering::SeqCst), 1);

        let payload = match Pin::new(&mut recv).poll(&mut cx) {
            Poll::Ready(Ok(payload)) => payload,
            other => panic!("expected ready payload after wake, got {other:?}"),
        };
        assert_eq!(payload.as_bytes(), &[0x2a]);
    }
}
