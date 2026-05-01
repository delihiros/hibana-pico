//! Host/full WASI P1 runner.
//!
//! This module is deliberately not part of the small Pico profile. It is the
//! larger-platform execution harness used to prove that ordinary Rust std
//! `wasm32-wasip1` artifacts enter the same typed syscall stream as embedded
//! guests. The Wasm core stays syscall-agnostic; this runner maps Preview 1
//! imports to `EngineReq`, drives the matching Engine/Kernel projected
//! localside exchange, and only then completes the guest through bounded
//! fd/lease/resource state.

extern crate std;

use std::vec::Vec;

use hibana::{
    Endpoint,
    g::{self, Msg, Role},
    substrate::{
        SessionKit,
        binding::NoBinding,
        ids::SessionId,
        program::{RoleProgram, project},
        runtime::{Config, CounterClock},
        tap::TapEvent,
    },
};

use crate::{
    choreography::protocol::{
        ArgsDone, ArgsGet, ArgsSizes, ArgsSizesGet, ClockNow, ClockResGet, ClockResolution,
        ClockTimeGet, EngineLabelUniverse, EngineReq, EngineRet, EnvironDone, EnvironGet,
        EnvironSizes, EnvironSizesGet, FdClosed, FdRead, FdReadDone, FdRequest, FdStat, FdWrite,
        FdWriteDone, LABEL_ENGINE_REQ, LABEL_ENGINE_RET, LABEL_GPIO_SET, LABEL_GPIO_SET_DONE,
        LABEL_TIMER_SLEEP_DONE, LABEL_TIMER_SLEEP_UNTIL, LABEL_WASI_ARGS_GET,
        LABEL_WASI_ARGS_GET_RET, LABEL_WASI_ARGS_SIZES_GET, LABEL_WASI_ARGS_SIZES_GET_RET,
        LABEL_WASI_CLOCK_RES_GET, LABEL_WASI_CLOCK_RES_GET_RET, LABEL_WASI_CLOCK_TIME_GET,
        LABEL_WASI_CLOCK_TIME_GET_RET, LABEL_WASI_ENVIRON_GET, LABEL_WASI_ENVIRON_GET_RET,
        LABEL_WASI_ENVIRON_SIZES_GET, LABEL_WASI_ENVIRON_SIZES_GET_RET, LABEL_WASI_FD_CLOSE,
        LABEL_WASI_FD_CLOSE_RET, LABEL_WASI_FD_FDSTAT_GET, LABEL_WASI_FD_FDSTAT_GET_RET,
        LABEL_WASI_FD_READ, LABEL_WASI_FD_READ_RET, LABEL_WASI_FD_WRITE, LABEL_WASI_FD_WRITE_RET,
        LABEL_WASI_PATH_OPEN, LABEL_WASI_PATH_OPEN_RET, LABEL_WASI_POLL_ONEOFF,
        LABEL_WASI_POLL_ONEOFF_RET, LABEL_WASI_PROC_EXIT, LABEL_WASI_RANDOM_GET,
        LABEL_WASI_RANDOM_GET_RET, LABEL_WASIP1_CLOCK_NOW, LABEL_WASIP1_CLOCK_NOW_RET,
        LABEL_WASIP1_EXIT, LABEL_WASIP1_RANDOM_SEED, LABEL_WASIP1_RANDOM_SEED_RET,
        LABEL_WASIP1_STDERR, LABEL_WASIP1_STDERR_RET, LABEL_WASIP1_STDIN, LABEL_WASIP1_STDIN_RET,
        LABEL_WASIP1_STDOUT, LABEL_WASIP1_STDOUT_RET, LABEL_YIELD_REQ, LABEL_YIELD_RET, MemRights,
        PollOneoff, PollReady, ProcExitStatus, RandomDone, RandomGet, WASIP1_STREAM_CHUNK_CAPACITY,
    },
    kernel::{
        choreofs::{
            CHOREOFS_WASI_ERRNO_NOSYS, ChoreoFsError, ChoreoFsObjectKind, ChoreoFsStat,
            ChoreoFsStore,
        },
        guest_ledger::{
            GuestFd, GuestLedger, GuestLedgerError, GuestQuotaLimits, WASI_ERRNO_BADF,
            WASI_ERRNO_INVAL, WASI_ERRNO_NOTCAPABLE, WASI_ERRNO_SUCCESS, WasiErrnoMap, WasiProfile,
        },
        wasi::{ChoreoResourceKind, PicoFdRights},
    },
    substrate::{exec::run_current_task, host_queue::HostQueueBackend, transport::SioTransport},
};

use super::wasm::{
    CoreWasip1FdStat, CoreWasip1FileStat, CoreWasip1Instance, CoreWasip1PathCall,
    CoreWasip1PathKind, CoreWasip1SocketKind, CoreWasip1Trap, CoreWasmMemoryGrow,
    WASIP1_FILETYPE_DIRECTORY, WASIP1_FILETYPE_REGULAR_FILE, WasmError,
};

const HOST_MEMORY_LEN: u32 = 2 * 1024 * 1024;
const HOST_MEMORY_EPOCH: u32 = 1;
const HOST_ROOT_FD: u8 = 3;
const HOST_FIRST_OBJECT_FD: u8 = 4;
const HOST_CLOCK_RESOLUTION_NANOS: u64 = 1_000_000;
const HOST_CLOCK_NOW_NANOS: u64 = 123_456_789;
const HOST_RANDOM_BYTE: u8 = 0x42;

pub type HostFullGuestLedger = GuestLedger<16, 8, 16>;
pub type HostFullChoreoFs = ChoreoFsStore<8, 64, 256>;
type HostFullTransport<'a> = SioTransport<&'a HostQueueBackend>;
type HostFullKit<'a> = SessionKit<'a, HostFullTransport<'a>, EngineLabelUniverse, CounterClock, 1>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NetworkAcceptRoute {
    listener_fd: u8,
    accepted_fd: u8,
    resource: ChoreoResourceKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreWasip1HostRunError {
    Wasm(WasmError),
    Ledger(GuestLedgerError),
    ChoreoFs(ChoreoFsError),
    PathRejected(u16),
    NetworkRejected(u16),
    Unsupported(&'static str),
    StepLimit,
}

impl From<WasmError> for CoreWasip1HostRunError {
    fn from(value: WasmError) -> Self {
        Self::Wasm(value)
    }
}

impl From<GuestLedgerError> for CoreWasip1HostRunError {
    fn from(value: GuestLedgerError) -> Self {
        Self::Ledger(value)
    }
}

impl From<ChoreoFsError> for CoreWasip1HostRunError {
    fn from(value: ChoreoFsError) -> Self {
        Self::ChoreoFs(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoreWasip1HostRunReport {
    pub exit_status: Option<u32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub engine_trace: Vec<EngineReq>,
    pub engine_replies: Vec<EngineRet>,
    pub memory_grow_count: u32,
    pub choreofs_open_count: u32,
    pub choreofs_read_count: u32,
    pub choreofs_write_count: u32,
    pub network_send_count: u32,
    pub network_recv_count: u32,
    pub network_accept_count: u32,
    pub network_accept_reject_count: u32,
    pub typed_reject_count: u32,
    pub localside_drive_count: u32,
}

pub struct CoreWasip1HostRunner<'a> {
    guest: CoreWasip1Instance<'a>,
    ledger: HostFullGuestLedger,
    fs: HostFullChoreoFs,
    fd_offsets: [(u8, u64); 16],
    network_rx: Vec<(u8, Vec<u8>)>,
    network_tx: Vec<(u8, Vec<u8>)>,
    network_accepts: Vec<NetworkAcceptRoute>,
    next_fd: u8,
    fail_closed_on_path_error: bool,
    fail_closed_on_network_error: bool,
}

impl<'a> CoreWasip1HostRunner<'a> {
    pub fn new(module: &'a [u8]) -> Result<Self, CoreWasip1HostRunError> {
        let mut ledger = GuestLedger::new(
            WasiProfile::HostFull,
            HOST_MEMORY_LEN,
            HOST_MEMORY_EPOCH,
            GuestQuotaLimits::new(16, 16),
            WasiErrnoMap::new(),
        );
        grant_stdio(&mut ledger)?;

        let fs = HostFullChoreoFs::new();
        fs.grant_preopen_root(&mut ledger, HOST_ROOT_FD)?;

        Ok(Self {
            guest: CoreWasip1Instance::new(
                module,
                crate::kernel::features::Wasip1HandlerSet::FULL,
            )?,
            ledger,
            fs,
            fd_offsets: [(0, 0); 16],
            network_rx: Vec::new(),
            network_tx: Vec::new(),
            network_accepts: Vec::new(),
            next_fd: HOST_FIRST_OBJECT_FD,
            fail_closed_on_path_error: false,
            fail_closed_on_network_error: false,
        })
    }

    pub fn guest(&self) -> &CoreWasip1Instance<'a> {
        &self.guest
    }

    pub fn fs_mut(&mut self) -> &mut HostFullChoreoFs {
        &mut self.fs
    }

    pub fn fail_closed_on_path_error(&mut self, enabled: bool) {
        self.fail_closed_on_path_error = enabled;
    }

    pub fn fail_closed_on_network_error(&mut self, enabled: bool) {
        self.fail_closed_on_network_error = enabled;
    }

    pub fn cap_grant_datagram(&mut self, fd: u8) -> Result<GuestFd, CoreWasip1HostRunError> {
        self.cap_grant_network(fd, ChoreoResourceKind::NetworkDatagram)
    }

    pub fn cap_grant_stream(&mut self, fd: u8) -> Result<GuestFd, CoreWasip1HostRunError> {
        self.cap_grant_network(fd, ChoreoResourceKind::NetworkStream)
    }

    pub fn cap_grant_listener(&mut self, fd: u8) -> Result<GuestFd, CoreWasip1HostRunError> {
        Ok(self.ledger.apply_fd_cap_grant(
            fd,
            PicoFdRights::Read,
            ChoreoResourceKind::NetworkListener,
            9,
            0,
            0,
            0,
            0,
            0,
            0,
        )?)
    }

    pub fn enqueue_datagram_accept(&mut self, listener_fd: u8, accepted_fd: u8) {
        self.network_accepts.push(NetworkAcceptRoute {
            listener_fd,
            accepted_fd,
            resource: ChoreoResourceKind::NetworkDatagram,
        });
    }

    pub fn enqueue_stream_accept(&mut self, listener_fd: u8, accepted_fd: u8) {
        self.network_accepts.push(NetworkAcceptRoute {
            listener_fd,
            accepted_fd,
            resource: ChoreoResourceKind::NetworkStream,
        });
    }

    pub fn enqueue_network_rx(&mut self, fd: u8, bytes: &[u8]) {
        self.network_rx.push((fd, bytes.to_vec()));
    }

    pub fn network_tx(&self) -> &[(u8, Vec<u8>)] {
        &self.network_tx
    }

    pub fn run_until_exit(
        &mut self,
        max_steps: usize,
    ) -> Result<CoreWasip1HostRunReport, CoreWasip1HostRunError> {
        let mut report = CoreWasip1HostRunReport::default();

        for _ in 0..max_steps {
            match self.guest.resume_with_fuel(250_000)? {
                CoreWasip1Trap::FdWrite(call) => {
                    let bytes = self.fd_write_bytes(call)?;
                    if bytes.is_empty() {
                        let request = EngineReq::FdWrite(
                            FdWrite::new_with_lease(call.fd(), 1, &[])
                                .map_err(|_| CoreWasip1HostRunError::Unsupported("fd_write"))?,
                        );
                        let reply = EngineRet::FdWriteDone(FdWriteDone::new(call.fd(), 0));
                        Self::drive_exchange(request, reply, &mut report)?;
                    } else {
                        for chunk in bytes.chunks(WASIP1_STREAM_CHUNK_CAPACITY) {
                            let request = EngineReq::FdWrite(
                                FdWrite::new_with_lease(call.fd(), 1, chunk)
                                    .map_err(|_| CoreWasip1HostRunError::Unsupported("fd_write"))?,
                            );
                            let reply = EngineRet::FdWriteDone(FdWriteDone::new(
                                call.fd(),
                                chunk.len().min(u8::MAX as usize) as u8,
                            ));
                            Self::drive_exchange(request, reply, &mut report)?;
                        }
                    }
                    match call.fd() {
                        1 => report.stdout.extend_from_slice(&bytes),
                        2 => report.stderr.extend_from_slice(&bytes),
                        fd => {
                            if self
                                .resolve_network_object(fd, PicoFdRights::Write)
                                .is_some()
                            {
                                self.network_tx.push((fd, bytes.clone()));
                                report.network_send_count =
                                    report.network_send_count.saturating_add(1);
                            } else if let Some(guest_fd) = self.resolve_object_fd(fd) {
                                self.fs
                                    .write(guest_fd, self.fd_offset(fd) as usize, &bytes)
                                    .map_err(CoreWasip1HostRunError::ChoreoFs)?;
                                self.advance_fd_offset(fd, bytes.len() as u64);
                                report.choreofs_write_count =
                                    report.choreofs_write_count.saturating_add(1);
                            }
                        }
                    }
                    self.guest
                        .complete_fd_write(call, WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::FdRead(call) => {
                    let fd = call.fd();
                    let max_len = self.fd_read_max_len(call)?;
                    let mut bytes = Vec::new();
                    if fd == 0 {
                        bytes.extend_from_slice(&[]);
                    } else if self
                        .resolve_network_object(fd, PicoFdRights::Read)
                        .is_some()
                    {
                        bytes = self.dequeue_network_rx(fd, max_len);
                        report.network_recv_count = report.network_recv_count.saturating_add(1);
                    } else if let Some(guest_fd) = self.resolve_object_fd(fd) {
                        let mut buf = [0u8; WASIP1_STREAM_CHUNK_CAPACITY];
                        let len = self
                            .fs
                            .read(
                                guest_fd,
                                self.fd_offset(fd) as usize,
                                &mut buf[..core::cmp::min(max_len, WASIP1_STREAM_CHUNK_CAPACITY)],
                            )
                            .map_err(CoreWasip1HostRunError::ChoreoFs)?;
                        bytes.extend_from_slice(&buf[..len]);
                        self.advance_fd_offset(fd, len as u64);
                        report.choreofs_read_count = report.choreofs_read_count.saturating_add(1);
                    }
                    let request = EngineReq::FdRead(
                        FdRead::new_with_lease(fd, 1, max_len.min(u8::MAX as usize) as u8)
                            .map_err(|_| CoreWasip1HostRunError::Unsupported("fd_read"))?,
                    );
                    let reply = EngineRet::FdReadDone(
                        FdReadDone::new_with_lease(fd, 1, &bytes)
                            .map_err(|_| CoreWasip1HostRunError::Unsupported("fd_read reply"))?,
                    );
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest
                        .complete_fd_read(call, &bytes, WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::FdFdstatGet(call) => {
                    let request = EngineReq::FdFdstatGet(FdRequest::new(call.fd()));
                    let rights = self.fd_rights(call.fd()).unwrap_or(MemRights::Read);
                    let reply = EngineRet::FdStat(FdStat::new(call.fd(), rights));
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest.complete_fd_fdstat_get(
                        call,
                        CoreWasip1FdStat::new(self.fd_filetype(call.fd()), 0, u64::MAX, u64::MAX),
                        WASI_ERRNO_SUCCESS as u32,
                    )?;
                }
                CoreWasip1Trap::FdClose(call) => {
                    let request = EngineReq::FdClose(FdRequest::new(call.fd()));
                    if call.fd() > 2 && call.fd() != HOST_ROOT_FD {
                        let _ = self.ledger.fd_view_mut().close_current(call.fd());
                    }
                    let reply = EngineRet::FdClosed(FdClosed::new(call.fd()));
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest.complete_host_call(WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::ClockResGet(call) => {
                    let request = EngineReq::ClockResGet(ClockResGet::new(call.clock_id() as u8));
                    let reply = EngineRet::ClockResolution(ClockResolution::new(
                        HOST_CLOCK_RESOLUTION_NANOS,
                    ));
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest.complete_clock_res_get(
                        call,
                        HOST_CLOCK_RESOLUTION_NANOS,
                        WASI_ERRNO_SUCCESS as u32,
                    )?;
                }
                CoreWasip1Trap::ClockTimeGet(call) => {
                    let request = EngineReq::ClockTimeGet(ClockTimeGet::new(
                        call.clock_id() as u8,
                        call.precision(),
                    ));
                    let reply = EngineRet::ClockTime(ClockNow::new(HOST_CLOCK_NOW_NANOS));
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest.complete_clock_time_get(
                        call,
                        HOST_CLOCK_NOW_NANOS,
                        WASI_ERRNO_SUCCESS as u32,
                    )?;
                }
                CoreWasip1Trap::PollOneoff(call) => {
                    let timeout = self.guest.poll_oneoff_delay_ticks(call).unwrap_or(0);
                    let request = EngineReq::PollOneoff(PollOneoff::new(timeout));
                    let reply = EngineRet::PollReady(PollReady::new(1));
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest
                        .complete_poll_oneoff(call, 1, WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::RandomGet(call) => {
                    let len = call.buf_len() as usize;
                    let bytes = std::vec![HOST_RANDOM_BYTE; len];
                    let request = EngineReq::RandomGet(
                        RandomGet::new_with_lease(1, len.min(WASIP1_STREAM_CHUNK_CAPACITY) as u8)
                            .map_err(|_| CoreWasip1HostRunError::Unsupported("random_get"))?,
                    );
                    let reply = EngineRet::RandomDone(
                        RandomDone::new_with_lease(
                            1,
                            &bytes[..core::cmp::min(bytes.len(), WASIP1_STREAM_CHUNK_CAPACITY)],
                        )
                        .map_err(|_| CoreWasip1HostRunError::Unsupported("random reply"))?,
                    );
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest
                        .complete_random_get(call, &bytes, WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::SchedYield => {
                    Self::drive_exchange(EngineReq::Yield, EngineRet::Yielded, &mut report)?;
                    self.guest.complete_sched_yield(WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::ArgsSizesGet(call) => {
                    Self::drive_exchange(
                        EngineReq::ArgsSizesGet(ArgsSizesGet::new()),
                        EngineRet::ArgsSizes(ArgsSizes::new(0, 0)),
                        &mut report,
                    )?;
                    self.guest
                        .complete_args_sizes_get(call, 0, 0, WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::ArgsGet(call) => {
                    let request = EngineReq::ArgsGet(
                        ArgsGet::new_with_lease(1, 0)
                            .map_err(|_| CoreWasip1HostRunError::Unsupported("args_get"))?,
                    );
                    let reply = EngineRet::ArgsDone(
                        ArgsDone::new_with_lease(1, &[])
                            .map_err(|_| CoreWasip1HostRunError::Unsupported("args reply"))?,
                    );
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest
                        .complete_args_get(call, &[], WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::EnvironSizesGet(call) => {
                    Self::drive_exchange(
                        EngineReq::EnvironSizesGet(EnvironSizesGet::new()),
                        EngineRet::EnvironSizes(EnvironSizes::new(0, 0)),
                        &mut report,
                    )?;
                    self.guest
                        .complete_environ_sizes_get(call, 0, 0, WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::EnvironGet(call) => {
                    let request = EngineReq::EnvironGet(
                        EnvironGet::new_with_lease(1, 0)
                            .map_err(|_| CoreWasip1HostRunError::Unsupported("environ_get"))?,
                    );
                    let reply = EngineRet::EnvironDone(
                        EnvironDone::new_with_lease(1, &[])
                            .map_err(|_| CoreWasip1HostRunError::Unsupported("environ reply"))?,
                    );
                    Self::drive_exchange(request, reply, &mut report)?;
                    self.guest
                        .complete_environ_get(call, &[], WASI_ERRNO_SUCCESS as u32)?;
                }
                CoreWasip1Trap::PathMinimal(call) => {
                    self.handle_path_minimal(call, &mut report)?;
                }
                CoreWasip1Trap::PathFull(call) => {
                    self.handle_path_full(call, &mut report)?;
                }
                CoreWasip1Trap::Socket(call) => {
                    self.handle_network_object_import(call, &mut report)?;
                }
                CoreWasip1Trap::ProcRaise(_) => {
                    self.guest.complete_proc_raise(WASI_ERRNO_INVAL as u32)?;
                }
                CoreWasip1Trap::ProcExit(status) => {
                    let request = EngineReq::ProcExit(ProcExitStatus::new(status as u8));
                    Self::drive_one_way(request, &mut report)?;
                    report.exit_status = Some(status);
                    return Ok(report);
                }
                CoreWasip1Trap::MemoryGrow(event) => {
                    self.handle_memory_grow(event, &mut report)?;
                }
                CoreWasip1Trap::Done => {
                    report.exit_status.get_or_insert(0);
                    return Ok(report);
                }
            }
        }

        Err(CoreWasip1HostRunError::StepLimit)
    }

    fn handle_path_minimal(
        &mut self,
        call: CoreWasip1PathCall,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        match call.kind() {
            CoreWasip1PathKind::FdPrestatGet => {
                let errno = if call.fd()? == HOST_ROOT_FD {
                    self.guest
                        .complete_fd_prestat_get(call, 1, WASI_ERRNO_SUCCESS as u32)?;
                    return Ok(());
                } else {
                    WASI_ERRNO_BADF
                };
                self.guest.complete_path_minimal(call, errno as u32)?;
            }
            CoreWasip1PathKind::FdPrestatDirName => {
                if call.fd()? == HOST_ROOT_FD {
                    self.guest.complete_fd_prestat_dir_name(
                        call,
                        b".",
                        WASI_ERRNO_SUCCESS as u32,
                    )?;
                } else {
                    self.guest
                        .complete_path_minimal(call, WASI_ERRNO_BADF as u32)?;
                }
            }
            CoreWasip1PathKind::PathOpen => {
                let path = self.guest.path_bytes(call)?;
                let new_fd = self.next_fd;
                let rights_base = call.arg_i64(5)?;
                let request = EngineReq::PathOpen(
                    crate::choreography::protocol::PathOpen::new(
                        call.fd()?,
                        0,
                        rights_base,
                        path.as_bytes(),
                    )
                    .map_err(|_| CoreWasip1HostRunError::Unsupported("path_open request"))?,
                );
                match self.fs.open_wasip1_path_with_ledger(
                    &mut self.ledger,
                    call.fd()?,
                    new_fd,
                    path.as_bytes(),
                    rights_base,
                ) {
                    Ok(_) => {
                        self.next_fd = self.next_fd.saturating_add(1);
                        report.choreofs_open_count = report.choreofs_open_count.saturating_add(1);
                        Self::drive_exchange(
                            request,
                            EngineRet::PathOpened(crate::choreography::protocol::PathOpened::new(
                                new_fd, 0,
                            )),
                            report,
                        )?;
                        self.guest.complete_path_open(
                            call,
                            new_fd as u32,
                            WASI_ERRNO_SUCCESS as u32,
                        )?;
                    }
                    Err(error) => {
                        let errno = error.wasi_errno();
                        report.typed_reject_count = report.typed_reject_count.saturating_add(1);
                        if self.fail_closed_on_path_error {
                            return Err(CoreWasip1HostRunError::PathRejected(errno));
                        }
                        self.guest.complete_path_open(call, 0, errno as u32)?;
                    }
                }
            }
            CoreWasip1PathKind::FdFilestatGet => {
                let stat = self.stat_fd(call.fd()?);
                self.guest
                    .complete_fd_filestat_get(call, stat, WASI_ERRNO_SUCCESS as u32)?;
            }
            CoreWasip1PathKind::PathFilestatGet => {
                let path = self.guest.path_bytes(call)?;
                match self.fs.stat_path(path.as_bytes()) {
                    Ok(stat) => self.guest.complete_path_filestat_get(
                        call,
                        core_stat_from_choreofs(stat),
                        WASI_ERRNO_SUCCESS as u32,
                    )?,
                    Err(error) => {
                        let errno = error.wasi_errno();
                        if self.fail_closed_on_path_error {
                            return Err(CoreWasip1HostRunError::PathRejected(errno));
                        }
                        self.guest.complete_path_minimal(call, errno as u32)?;
                    }
                }
            }
            CoreWasip1PathKind::FdReaddir => {
                self.guest
                    .complete_fd_readdir(call, &[], WASI_ERRNO_SUCCESS as u32)?;
            }
            CoreWasip1PathKind::PathReadlink
            | CoreWasip1PathKind::PathCreateDirectory
            | CoreWasip1PathKind::PathRemoveDirectory
            | CoreWasip1PathKind::PathUnlinkFile
            | CoreWasip1PathKind::PathRename => {
                report.typed_reject_count = report.typed_reject_count.saturating_add(1);
                self.guest
                    .complete_path_minimal(call, CHOREOFS_WASI_ERRNO_NOSYS as u32)?;
            }
            _ => {
                report.typed_reject_count = report.typed_reject_count.saturating_add(1);
                self.guest
                    .complete_path_minimal(call, CHOREOFS_WASI_ERRNO_NOSYS as u32)?;
            }
        }
        Ok(())
    }

    fn handle_path_full(
        &mut self,
        call: CoreWasip1PathCall,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        match call.kind() {
            CoreWasip1PathKind::FdSeek => {
                let fd = call.fd()?;
                let offset = call.arg_i64(1)?;
                self.set_fd_offset(fd, offset);
                self.guest
                    .complete_fd_seek(call, offset, WASI_ERRNO_SUCCESS as u32)?;
            }
            CoreWasip1PathKind::FdTell => {
                self.guest.complete_fd_tell(
                    call,
                    self.fd_offset(call.fd()?),
                    WASI_ERRNO_SUCCESS as u32,
                )?;
            }
            CoreWasip1PathKind::FdPread => {
                let fd = call.fd()?;
                let guest_fd = self
                    .resolve_object_fd(fd)
                    .ok_or(CoreWasip1HostRunError::PathRejected(WASI_ERRNO_BADF))?;
                let mut buf = [0u8; WASIP1_STREAM_CHUNK_CAPACITY];
                let offset = call.arg_i64(3)? as usize;
                let len = self.fs.read(guest_fd, offset, &mut buf)?;
                self.guest
                    .complete_fd_pread(call, &buf[..len], WASI_ERRNO_SUCCESS as u32)?;
            }
            CoreWasip1PathKind::FdPwrite => {
                report.typed_reject_count = report.typed_reject_count.saturating_add(1);
                self.guest
                    .complete_fd_pwrite(call, 0, CHOREOFS_WASI_ERRNO_NOSYS as u32)?;
            }
            CoreWasip1PathKind::FdSync
            | CoreWasip1PathKind::FdDatasync
            | CoreWasip1PathKind::FdAdvise
            | CoreWasip1PathKind::FdAllocate
            | CoreWasip1PathKind::FdFdstatSetFlags
            | CoreWasip1PathKind::FdFdstatSetRights
            | CoreWasip1PathKind::FdFilestatSetSize
            | CoreWasip1PathKind::FdFilestatSetTimes
            | CoreWasip1PathKind::FdRenumber
            | CoreWasip1PathKind::PathFilestatSetTimes
            | CoreWasip1PathKind::PathLink
            | CoreWasip1PathKind::PathSymlink => {
                report.typed_reject_count = report.typed_reject_count.saturating_add(1);
                self.guest
                    .complete_path_full(call, CHOREOFS_WASI_ERRNO_NOSYS as u32)?;
            }
            _ => {
                report.typed_reject_count = report.typed_reject_count.saturating_add(1);
                self.guest
                    .complete_path_full(call, CHOREOFS_WASI_ERRNO_NOSYS as u32)?;
            }
        }
        Ok(())
    }

    fn handle_network_object_import(
        &mut self,
        call: super::wasm::CoreWasip1SocketCall,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        // WASI Preview 1 names these imports `sock_*`, but hibana-pico does
        // not give sockets independent semantic authority. They normalize into
        // the same NetworkObject fd-write/fd-read/fd-close stream used by the
        // choreography.
        match call.kind() {
            CoreWasip1SocketKind::SockSend => {
                let fd = call.fd()? as u8;
                if self
                    .resolve_network_object(fd, PicoFdRights::Write)
                    .is_none()
                {
                    return self.reject_network_object_import(call, WASI_ERRNO_NOTCAPABLE, report);
                }
                let request = self.guest.socket_as_engine_req(call, 1)?;
                let payload = self.guest.sock_send_payload(call)?;
                self.network_tx.push((fd, payload.as_bytes().to_vec()));
                report.network_send_count = report.network_send_count.saturating_add(1);
                let reply = EngineRet::FdWriteDone(FdWriteDone::new(
                    fd,
                    payload.as_bytes().len().min(u8::MAX as usize) as u8,
                ));
                Self::drive_exchange(request, reply, report)?;
                self.guest.complete_sock_send(
                    call,
                    payload.as_bytes().len() as u32,
                    WASI_ERRNO_SUCCESS as u32,
                )?;
            }
            CoreWasip1SocketKind::SockRecv => {
                let fd = call.fd()? as u8;
                if self
                    .resolve_network_object(fd, PicoFdRights::Read)
                    .is_none()
                {
                    return self.reject_network_object_import(call, WASI_ERRNO_NOTCAPABLE, report);
                }
                let request = self.guest.socket_as_engine_req(call, 1)?;
                let (_, max_len) = self.guest.sock_recv_iovec(call)?;
                let bytes = self.dequeue_network_rx(fd, max_len as usize);
                report.network_recv_count = report.network_recv_count.saturating_add(1);
                let reply = EngineRet::FdReadDone(
                    FdReadDone::new_with_lease(fd, 1, &bytes)
                        .map_err(|_| CoreWasip1HostRunError::Unsupported("sock_recv reply"))?,
                );
                Self::drive_exchange(request, reply, report)?;
                self.guest
                    .complete_sock_recv(call, &bytes, 0, WASI_ERRNO_SUCCESS as u32)?;
            }
            CoreWasip1SocketKind::SockShutdown => {
                let fd = call.fd()? as u8;
                if self
                    .resolve_network_object(fd, PicoFdRights::Read)
                    .is_none()
                    && self
                        .resolve_network_object(fd, PicoFdRights::Write)
                        .is_none()
                {
                    return self.reject_network_object_import(call, WASI_ERRNO_NOTCAPABLE, report);
                }
                let request = EngineReq::FdClose(FdRequest::new(fd));
                let reply = EngineRet::FdClosed(FdClosed::new(fd));
                Self::drive_exchange(request, reply, report)?;
                let _ = self.ledger.fd_view_mut().close_current(fd);
                self.guest
                    .complete_sock_shutdown(call, WASI_ERRNO_SUCCESS as u32)?;
            }
            CoreWasip1SocketKind::SockAccept => {
                let listener_fd = call.fd()? as u8;
                if let Err(error) = self.ledger.resolve_fd(
                    listener_fd,
                    PicoFdRights::Read,
                    ChoreoResourceKind::NetworkListener,
                ) {
                    return self.reject_network_object_import(
                        call,
                        self.ledger.errno(error),
                        report,
                    );
                }
                let Some(route_index) = self
                    .network_accepts
                    .iter()
                    .position(|route| route.listener_fd == listener_fd)
                else {
                    report.network_accept_reject_count =
                        report.network_accept_reject_count.saturating_add(1);
                    return self.reject_network_object_import(
                        call,
                        CHOREOFS_WASI_ERRNO_NOSYS,
                        report,
                    );
                };
                let route = self.network_accepts.remove(route_index);
                let token = match self.ledger.begin_sock_accept(listener_fd) {
                    Ok(token) => token,
                    Err(error) => {
                        return self.reject_network_object_import(
                            call,
                            self.ledger.errno(error),
                            report,
                        );
                    }
                };
                let accepted = match self.cap_mint_network(route.accepted_fd, route.resource) {
                    Ok(accepted) => accepted,
                    Err(CoreWasip1HostRunError::Ledger(error)) => {
                        return self.reject_network_object_import(
                            call,
                            self.ledger.errno(error),
                            report,
                        );
                    }
                    Err(error) => return Err(error),
                };
                if let Err(error) = self.ledger.complete_sock_accept(token, listener_fd) {
                    return self.reject_network_object_import(
                        call,
                        self.ledger.errno(error),
                        report,
                    );
                }
                report.network_accept_count = report.network_accept_count.saturating_add(1);
                self.guest.complete_sock_accept(
                    call,
                    accepted.fd() as u32,
                    WASI_ERRNO_SUCCESS as u32,
                )?;
            }
        }
        Ok(())
    }

    fn reject_network_object_import(
        &mut self,
        call: super::wasm::CoreWasip1SocketCall,
        errno: u16,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        report.typed_reject_count = report.typed_reject_count.saturating_add(1);
        if self.fail_closed_on_network_error {
            return Err(CoreWasip1HostRunError::NetworkRejected(errno));
        }
        self.guest.complete_socket(call, errno as u32)?;
        Ok(())
    }

    fn handle_memory_grow(
        &mut self,
        _event: CoreWasmMemoryGrow,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        let _ = self.guest.complete_memory_grow_event()?;
        report.memory_grow_count = report.memory_grow_count.saturating_add(1);
        Ok(())
    }

    fn drive_exchange(
        request: EngineReq,
        reply: EngineRet,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        match (request, reply) {
            (EngineReq::LogU32(_), EngineRet::Logged(_)) => {
                Self::drive_pair::<LABEL_ENGINE_REQ, LABEL_ENGINE_RET>(request, reply, report)
            }
            (EngineReq::Yield, EngineRet::Yielded) => {
                Self::drive_pair::<LABEL_YIELD_REQ, LABEL_YIELD_RET>(request, reply, report)
            }
            (EngineReq::Wasip1Stdout(_), EngineRet::Wasip1StdoutWritten(_)) => {
                Self::drive_pair::<LABEL_WASIP1_STDOUT, LABEL_WASIP1_STDOUT_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::Wasip1Stderr(_), EngineRet::Wasip1StderrWritten(_)) => {
                Self::drive_pair::<LABEL_WASIP1_STDERR, LABEL_WASIP1_STDERR_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::Wasip1Stdin(_), EngineRet::Wasip1StdinRead(_)) => {
                Self::drive_pair::<LABEL_WASIP1_STDIN, LABEL_WASIP1_STDIN_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::Wasip1ClockNow, EngineRet::Wasip1ClockNow(_)) => {
                Self::drive_pair::<LABEL_WASIP1_CLOCK_NOW, LABEL_WASIP1_CLOCK_NOW_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::Wasip1RandomSeed, EngineRet::Wasip1RandomSeed(_)) => {
                Self::drive_pair::<LABEL_WASIP1_RANDOM_SEED, LABEL_WASIP1_RANDOM_SEED_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::FdWrite(_), EngineRet::FdWriteDone(_)) => {
                Self::drive_pair::<LABEL_WASI_FD_WRITE, LABEL_WASI_FD_WRITE_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::FdRead(_), EngineRet::FdReadDone(_)) => {
                Self::drive_pair::<LABEL_WASI_FD_READ, LABEL_WASI_FD_READ_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::FdFdstatGet(_), EngineRet::FdStat(_)) => {
                Self::drive_pair::<LABEL_WASI_FD_FDSTAT_GET, LABEL_WASI_FD_FDSTAT_GET_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::FdClose(_), EngineRet::FdClosed(_)) => Self::drive_pair::<
                LABEL_WASI_FD_CLOSE,
                LABEL_WASI_FD_CLOSE_RET,
            >(request, reply, report),
            (EngineReq::ClockResGet(_), EngineRet::ClockResolution(_)) => {
                Self::drive_pair::<LABEL_WASI_CLOCK_RES_GET, LABEL_WASI_CLOCK_RES_GET_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::ClockTimeGet(_), EngineRet::ClockTime(_)) => {
                Self::drive_pair::<LABEL_WASI_CLOCK_TIME_GET, LABEL_WASI_CLOCK_TIME_GET_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::PollOneoff(_), EngineRet::PollReady(_)) => {
                Self::drive_pair::<LABEL_WASI_POLL_ONEOFF, LABEL_WASI_POLL_ONEOFF_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::RandomGet(_), EngineRet::RandomDone(_)) => {
                Self::drive_pair::<LABEL_WASI_RANDOM_GET, LABEL_WASI_RANDOM_GET_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::ArgsSizesGet(_), EngineRet::ArgsSizes(_)) => {
                Self::drive_pair::<LABEL_WASI_ARGS_SIZES_GET, LABEL_WASI_ARGS_SIZES_GET_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::ArgsGet(_), EngineRet::ArgsDone(_)) => Self::drive_pair::<
                LABEL_WASI_ARGS_GET,
                LABEL_WASI_ARGS_GET_RET,
            >(request, reply, report),
            (EngineReq::EnvironSizesGet(_), EngineRet::EnvironSizes(_)) => {
                Self::drive_pair::<LABEL_WASI_ENVIRON_SIZES_GET, LABEL_WASI_ENVIRON_SIZES_GET_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::EnvironGet(_), EngineRet::EnvironDone(_)) => {
                Self::drive_pair::<LABEL_WASI_ENVIRON_GET, LABEL_WASI_ENVIRON_GET_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::PathOpen(_), EngineRet::PathOpened(_)) => {
                Self::drive_pair::<LABEL_WASI_PATH_OPEN, LABEL_WASI_PATH_OPEN_RET>(
                    request, reply, report,
                )
            }
            (EngineReq::TimerSleepUntil(_), EngineRet::TimerSleepDone(_)) => {
                Self::drive_pair::<LABEL_TIMER_SLEEP_UNTIL, LABEL_TIMER_SLEEP_DONE>(
                    request, reply, report,
                )
            }
            (EngineReq::GpioSet(_), EngineRet::GpioSetDone(_)) => {
                Self::drive_pair::<LABEL_GPIO_SET, LABEL_GPIO_SET_DONE>(request, reply, report)
            }
            _ => Err(CoreWasip1HostRunError::Unsupported(
                "host/full localside request/reply mismatch",
            )),
        }
    }

    fn drive_one_way(
        request: EngineReq,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        match request {
            EngineReq::ProcExit(_) => {
                Self::drive_send_only::<LABEL_WASI_PROC_EXIT>(request, report)
            }
            EngineReq::Wasip1Exit(_) => Self::drive_send_only::<LABEL_WASIP1_EXIT>(request, report),
            _ => Err(CoreWasip1HostRunError::Unsupported(
                "host/full localside one-way request mismatch",
            )),
        }
    }

    fn drive_pair<const REQ_LABEL: u8, const RET_LABEL: u8>(
        request: EngineReq,
        reply: EngineRet,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        let program = g::seq(
            g::send::<Role<1>, Role<0>, Msg<REQ_LABEL, EngineReq>, 1>(),
            g::send::<Role<0>, Role<1>, Msg<RET_LABEL, EngineRet>, 1>(),
        );
        let kernel_program: RoleProgram<0> = project(&program);
        let engine_program: RoleProgram<1> = project(&program);
        Self::drive_projected_pair::<REQ_LABEL, RET_LABEL>(
            &kernel_program,
            &engine_program,
            request,
            reply,
            report,
        )?;
        report.engine_trace.push(request);
        report.engine_replies.push(reply);
        report.localside_drive_count = report.localside_drive_count.saturating_add(1);
        Ok(())
    }

    fn drive_send_only<const REQ_LABEL: u8>(
        request: EngineReq,
        report: &mut CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        let program = g::send::<Role<1>, Role<0>, Msg<REQ_LABEL, EngineReq>, 1>();
        let kernel_program: RoleProgram<0> = project(&program);
        let engine_program: RoleProgram<1> = project(&program);
        Self::drive_projected_send::<REQ_LABEL>(&kernel_program, &engine_program, request, report)?;
        report.engine_trace.push(request);
        report.localside_drive_count = report.localside_drive_count.saturating_add(1);
        Ok(())
    }

    fn drive_projected_pair<const REQ_LABEL: u8, const RET_LABEL: u8>(
        kernel_program: &RoleProgram<0>,
        engine_program: &RoleProgram<1>,
        request: EngineReq,
        reply: EngineRet,
        report: &CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        let backend = HostQueueBackend::new();
        let clock = CounterClock::new();
        let mut tap = [TapEvent::zero(); 128];
        let mut slab = std::vec![0u8; 192 * 1024];
        let cluster = HostFullKit::new(&clock);
        let rv = cluster
            .add_rendezvous_from_config(
                Config::new(&mut tap, slab.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full localside rendezvous"))?;
        let sid = SessionId::new(40_000u32.saturating_add(report.localside_drive_count));
        let mut kernel: Endpoint<'_, 0> = cluster
            .enter(rv, sid, kernel_program, NoBinding)
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full localside kernel"))?;
        let mut engine: Endpoint<'_, 1> = cluster
            .enter(rv, sid, engine_program, NoBinding)
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full localside engine"))?;

        run_current_task(async {
            (engine
                .flow::<Msg<REQ_LABEL, EngineReq>>()
                .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full request phase"))?
                .send(&request))
            .await
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full request send"))?;

            let received = (kernel.recv::<Msg<REQ_LABEL, EngineReq>>())
                .await
                .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full request recv"))?;
            if received != request {
                return Err(CoreWasip1HostRunError::Unsupported(
                    "host/full request decode mismatch",
                ));
            }

            (kernel
                .flow::<Msg<RET_LABEL, EngineRet>>()
                .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full reply phase"))?
                .send(&reply))
            .await
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full reply send"))?;

            let received = (engine.recv::<Msg<RET_LABEL, EngineRet>>())
                .await
                .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full reply recv"))?;
            if received != reply {
                return Err(CoreWasip1HostRunError::Unsupported(
                    "host/full reply decode mismatch",
                ));
            }

            Ok(())
        })
    }

    fn drive_projected_send<const REQ_LABEL: u8>(
        kernel_program: &RoleProgram<0>,
        engine_program: &RoleProgram<1>,
        request: EngineReq,
        report: &CoreWasip1HostRunReport,
    ) -> Result<(), CoreWasip1HostRunError> {
        let backend = HostQueueBackend::new();
        let clock = CounterClock::new();
        let mut tap = [TapEvent::zero(); 128];
        let mut slab = std::vec![0u8; 192 * 1024];
        let cluster = HostFullKit::new(&clock);
        let rv = cluster
            .add_rendezvous_from_config(
                Config::new(&mut tap, slab.as_mut_slice()).with_universe(EngineLabelUniverse),
                SioTransport::new(&backend),
            )
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full localside rendezvous"))?;
        let sid = SessionId::new(50_000u32.saturating_add(report.localside_drive_count));
        let mut kernel: Endpoint<'_, 0> = cluster
            .enter(rv, sid, kernel_program, NoBinding)
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full localside kernel"))?;
        let mut engine: Endpoint<'_, 1> = cluster
            .enter(rv, sid, engine_program, NoBinding)
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full localside engine"))?;

        run_current_task(async {
            (engine
                .flow::<Msg<REQ_LABEL, EngineReq>>()
                .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full request phase"))?
                .send(&request))
            .await
            .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full request send"))?;

            let received = (kernel.recv::<Msg<REQ_LABEL, EngineReq>>())
                .await
                .map_err(|_| CoreWasip1HostRunError::Unsupported("host/full request recv"))?;
            if received != request {
                return Err(CoreWasip1HostRunError::Unsupported(
                    "host/full request decode mismatch",
                ));
            }

            Ok(())
        })
    }

    fn fd_write_bytes(
        &self,
        call: super::wasm::TinyWasip1FdWriteCall,
    ) -> Result<Vec<u8>, CoreWasip1HostRunError> {
        let total = self.guest.fd_write_total_len(call)? as usize;
        let mut bytes = Vec::with_capacity(total);
        if call.iovs_len() == 0 {
            bytes.resize(total, 0);
            self.guest.read_memory(call.iovs(), &mut bytes)?;
            return Ok(bytes);
        }
        for index in 0..call.iovs_len() {
            let iov = call
                .iovs()
                .checked_add(index.saturating_mul(8))
                .ok_or(CoreWasip1HostRunError::Wasm(WasmError::Truncated))?;
            let ptr = self.guest.read_memory_u32(iov)?;
            let len = self.guest.read_memory_u32(iov.saturating_add(4))? as usize;
            let start = bytes.len();
            bytes.resize(start + len, 0);
            self.guest.read_memory(ptr, &mut bytes[start..])?;
        }
        Ok(bytes)
    }

    fn fd_read_max_len(
        &self,
        call: super::wasm::CoreWasip1FdReadCall,
    ) -> Result<usize, CoreWasip1HostRunError> {
        if call.iovs_len() == 0 {
            return Ok(0);
        }
        let mut total = 0usize;
        for index in 0..call.iovs_len() {
            let iov = call
                .iovs()
                .checked_add(index.saturating_mul(8))
                .ok_or(CoreWasip1HostRunError::Wasm(WasmError::Truncated))?;
            total =
                total.saturating_add(self.guest.read_memory_u32(iov.saturating_add(4))? as usize);
        }
        Ok(total.min(WASIP1_STREAM_CHUNK_CAPACITY))
    }

    fn resolve_object_fd(&self, fd: u8) -> Option<GuestFd> {
        self.ledger
            .resolve_fd(fd, PicoFdRights::Read, ChoreoResourceKind::ChoreoObject)
            .ok()
            .or_else(|| {
                self.ledger
                    .resolve_fd(fd, PicoFdRights::Write, ChoreoResourceKind::ChoreoObject)
                    .ok()
            })
            .or_else(|| {
                self.ledger
                    .resolve_fd(
                        fd,
                        PicoFdRights::ReadWrite,
                        ChoreoResourceKind::ChoreoObject,
                    )
                    .ok()
            })
    }

    fn resolve_network_object(&self, fd: u8, rights: PicoFdRights) -> Option<GuestFd> {
        self.ledger
            .resolve_fd(fd, rights, ChoreoResourceKind::NetworkDatagram)
            .ok()
            .or_else(|| {
                self.ledger
                    .resolve_fd(fd, rights, ChoreoResourceKind::NetworkStream)
                    .ok()
            })
    }

    fn dequeue_network_rx(&mut self, fd: u8, max_len: usize) -> Vec<u8> {
        let Some(index) = self
            .network_rx
            .iter()
            .position(|(slot_fd, _)| *slot_fd == fd)
        else {
            return Vec::new();
        };
        let (_, mut bytes) = self.network_rx.remove(index);
        if bytes.len() <= max_len {
            return bytes;
        }
        let tail = bytes.split_off(max_len);
        self.network_rx.insert(0, (fd, tail));
        bytes
    }

    fn fd_rights(&self, fd: u8) -> Option<MemRights> {
        if self
            .ledger
            .resolve_fd(fd, PicoFdRights::Write, ChoreoResourceKind::Stdout)
            .is_ok()
            || self
                .ledger
                .resolve_fd(fd, PicoFdRights::Write, ChoreoResourceKind::Stderr)
                .is_ok()
        {
            return Some(MemRights::Read);
        }
        if self
            .ledger
            .resolve_fd(fd, PicoFdRights::Read, ChoreoResourceKind::Stdin)
            .is_ok()
            || self.resolve_object_fd(fd).is_some()
        {
            return Some(MemRights::Read);
        }
        None
    }

    fn fd_filetype(&self, fd: u8) -> u8 {
        if fd == HOST_ROOT_FD
            || self
                .ledger
                .resolve_fd(fd, PicoFdRights::Read, ChoreoResourceKind::DirectoryView)
                .is_ok()
        {
            WASIP1_FILETYPE_DIRECTORY
        } else {
            WASIP1_FILETYPE_REGULAR_FILE
        }
    }

    fn stat_fd(&self, fd: u8) -> CoreWasip1FileStat {
        if fd == HOST_ROOT_FD {
            return CoreWasip1FileStat::new(WASIP1_FILETYPE_DIRECTORY, 0);
        }
        if let Some(guest_fd) = self.resolve_object_fd(fd) {
            if let Ok(stat) = self.fs.stat_fd(guest_fd) {
                return core_stat_from_choreofs(stat);
            }
        }
        CoreWasip1FileStat::new(WASIP1_FILETYPE_REGULAR_FILE, 0)
    }

    fn fd_offset(&self, fd: u8) -> u64 {
        self.fd_offsets
            .iter()
            .find_map(|(slot_fd, offset)| (*slot_fd == fd).then_some(*offset))
            .unwrap_or(0)
    }

    fn set_fd_offset(&mut self, fd: u8, offset: u64) {
        if let Some(slot) = self.fd_offsets.iter_mut().find(|slot| slot.0 == fd) {
            slot.1 = offset;
            return;
        }
        if let Some(slot) = self.fd_offsets.iter_mut().find(|slot| slot.0 == 0) {
            *slot = (fd, offset);
        }
    }

    fn advance_fd_offset(&mut self, fd: u8, delta: u64) {
        self.set_fd_offset(fd, self.fd_offset(fd).saturating_add(delta));
    }

    fn cap_grant_network(
        &mut self,
        fd: u8,
        resource: ChoreoResourceKind,
    ) -> Result<GuestFd, CoreWasip1HostRunError> {
        Ok(self.ledger.apply_fd_cap_grant(
            fd,
            PicoFdRights::ReadWrite,
            resource,
            9,
            0,
            0,
            0,
            0,
            0,
            0,
        )?)
    }

    fn cap_mint_network(
        &mut self,
        fd: u8,
        resource: ChoreoResourceKind,
    ) -> Result<GuestFd, CoreWasip1HostRunError> {
        Ok(self.ledger.apply_fd_cap_mint(
            fd,
            PicoFdRights::ReadWrite,
            resource,
            9,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        )?)
    }
}

fn grant_stdio(ledger: &mut HostFullGuestLedger) -> Result<(), GuestLedgerError> {
    ledger.apply_fd_cap_grant(
        0,
        PicoFdRights::Read,
        ChoreoResourceKind::Stdin,
        1,
        0,
        0,
        0,
        0,
        0,
        0,
    )?;
    ledger.apply_fd_cap_grant(
        1,
        PicoFdRights::Write,
        ChoreoResourceKind::Stdout,
        1,
        0,
        0,
        0,
        0,
        0,
        0,
    )?;
    ledger.apply_fd_cap_grant(
        2,
        PicoFdRights::Write,
        ChoreoResourceKind::Stderr,
        1,
        0,
        0,
        0,
        0,
        0,
        0,
    )?;
    Ok(())
}

fn core_stat_from_choreofs(stat: ChoreoFsStat) -> CoreWasip1FileStat {
    let filetype = match stat.kind() {
        ChoreoFsObjectKind::Directory => WASIP1_FILETYPE_DIRECTORY,
        _ => WASIP1_FILETYPE_REGULAR_FILE,
    };
    CoreWasip1FileStat::new(filetype, stat.size() as u64)
}
