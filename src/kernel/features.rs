#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasmEngineProfile {
    None,
    Tiny,
    /// Core Wasm execution capacity only. This does not imply any WASI P1
    /// syscall handler; syscall imports are selected by `wasip1-sys-*`.
    Core,
    Wasip1StdProfile,
    Wasip1Full,
}

impl WasmEngineProfile {
    pub const fn active() -> Self {
        if cfg!(feature = "wasm-engine-wasip1-full") {
            Self::Wasip1Full
        } else if cfg!(feature = "wasm-engine-wasip1-std-profile") {
            Self::Wasip1StdProfile
        } else if cfg!(feature = "wasm-engine-core") {
            Self::Core
        } else if cfg!(feature = "wasm-engine-tiny") {
            Self::Tiny
        } else {
            Self::None
        }
    }

    pub const fn can_run_ordinary_wasip1_std(self) -> bool {
        matches!(self, Self::Wasip1Full)
    }

    pub const fn can_run_core_wasip1(self) -> bool {
        matches!(self, Self::Core | Self::Wasip1StdProfile | Self::Wasip1Full)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wasip1Syscall {
    ArgsEnv,
    FdWrite,
    FdRead,
    FdFdstatGet,
    FdClose,
    ClockResGet,
    ClockTimeGet,
    PollOneoff,
    RandomGet,
    ProcExit,
    ProcRaise,
    SchedYield,
    PathMinimal,
    PathFull,
    NetworkObject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wasip1ImportDisposition {
    Supported,
    TypedEnosys,
    TypedReject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wasip1ImportEffectiveDisposition {
    Supported,
    TypedEnosys,
    TypedReject,
    UnsupportedByProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wasip1ImportCoverage {
    pub import: &'static str,
    pub syscall: Wasip1Syscall,
    pub disposition: Wasip1ImportDisposition,
}

impl Wasip1ImportCoverage {
    pub const fn effective(self, handlers: Wasip1HandlerSet) -> Wasip1ImportEffectiveDisposition {
        if !handlers.supports(self.syscall) {
            return Wasip1ImportEffectiveDisposition::UnsupportedByProfile;
        }
        match self.disposition {
            Wasip1ImportDisposition::Supported => Wasip1ImportEffectiveDisposition::Supported,
            Wasip1ImportDisposition::TypedEnosys => Wasip1ImportEffectiveDisposition::TypedEnosys,
            Wasip1ImportDisposition::TypedReject => Wasip1ImportEffectiveDisposition::TypedReject,
        }
    }
}

pub const WASIP1_PREVIEW1_IMPORT_COVERAGE: [Wasip1ImportCoverage; 46] = [
    Wasip1ImportCoverage {
        import: "args_get",
        syscall: Wasip1Syscall::ArgsEnv,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "args_sizes_get",
        syscall: Wasip1Syscall::ArgsEnv,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "clock_res_get",
        syscall: Wasip1Syscall::ClockResGet,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "clock_time_get",
        syscall: Wasip1Syscall::ClockTimeGet,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "environ_get",
        syscall: Wasip1Syscall::ArgsEnv,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "environ_sizes_get",
        syscall: Wasip1Syscall::ArgsEnv,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_advise",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_allocate",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_close",
        syscall: Wasip1Syscall::FdClose,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_datasync",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_fdstat_get",
        syscall: Wasip1Syscall::FdFdstatGet,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_fdstat_set_flags",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_fdstat_set_rights",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_filestat_get",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_filestat_set_size",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_filestat_set_times",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_pread",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_prestat_get",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_prestat_dir_name",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_pwrite",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_read",
        syscall: Wasip1Syscall::FdRead,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_readdir",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_renumber",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_seek",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_sync",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "fd_tell",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "fd_write",
        syscall: Wasip1Syscall::FdWrite,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "path_create_directory",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "path_filestat_get",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "path_filestat_set_times",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "path_link",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "path_open",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "path_readlink",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "path_remove_directory",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "path_rename",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "path_symlink",
        syscall: Wasip1Syscall::PathFull,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "path_unlink_file",
        syscall: Wasip1Syscall::PathMinimal,
        disposition: Wasip1ImportDisposition::TypedEnosys,
    },
    Wasip1ImportCoverage {
        import: "poll_oneoff",
        syscall: Wasip1Syscall::PollOneoff,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "proc_exit",
        syscall: Wasip1Syscall::ProcExit,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "proc_raise",
        syscall: Wasip1Syscall::ProcRaise,
        disposition: Wasip1ImportDisposition::TypedReject,
    },
    Wasip1ImportCoverage {
        import: "random_get",
        syscall: Wasip1Syscall::RandomGet,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "sched_yield",
        syscall: Wasip1Syscall::SchedYield,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "sock_accept",
        syscall: Wasip1Syscall::NetworkObject,
        disposition: Wasip1ImportDisposition::TypedReject,
    },
    Wasip1ImportCoverage {
        import: "sock_recv",
        syscall: Wasip1Syscall::NetworkObject,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "sock_send",
        syscall: Wasip1Syscall::NetworkObject,
        disposition: Wasip1ImportDisposition::Supported,
    },
    Wasip1ImportCoverage {
        import: "sock_shutdown",
        syscall: Wasip1Syscall::NetworkObject,
        disposition: Wasip1ImportDisposition::Supported,
    },
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wasip1HandlerSet {
    pub args_env: bool,
    pub fd_write: bool,
    pub fd_read: bool,
    pub fd_fdstat_get: bool,
    pub fd_close: bool,
    pub clock_res_get: bool,
    pub clock_time_get: bool,
    pub poll_oneoff: bool,
    pub random_get: bool,
    pub proc_exit: bool,
    pub proc_raise: bool,
    pub sched_yield: bool,
    pub path_minimal: bool,
    pub path_full: bool,
    pub network_object: bool,
}

impl Wasip1HandlerSet {
    pub const EMPTY: Self = Self {
        args_env: false,
        fd_write: false,
        fd_read: false,
        fd_fdstat_get: false,
        fd_close: false,
        clock_res_get: false,
        clock_time_get: false,
        poll_oneoff: false,
        random_get: false,
        proc_exit: false,
        proc_raise: false,
        sched_yield: false,
        path_minimal: false,
        path_full: false,
        network_object: false,
    };

    pub const BAKER_MIN: Self = Self {
        args_env: false,
        fd_write: true,
        fd_read: false,
        fd_fdstat_get: false,
        fd_close: false,
        clock_res_get: false,
        clock_time_get: false,
        poll_oneoff: true,
        random_get: false,
        proc_exit: true,
        proc_raise: false,
        sched_yield: false,
        path_minimal: false,
        path_full: false,
        network_object: false,
    };

    pub const FULL: Self = Self {
        args_env: true,
        fd_write: true,
        fd_read: true,
        fd_fdstat_get: true,
        fd_close: true,
        clock_res_get: true,
        clock_time_get: true,
        poll_oneoff: true,
        random_get: true,
        proc_exit: true,
        proc_raise: true,
        sched_yield: true,
        path_minimal: true,
        path_full: true,
        network_object: true,
    };

    pub const fn active() -> Self {
        Self {
            args_env: cfg!(feature = "wasip1-sys-args-env"),
            fd_write: cfg!(feature = "wasip1-sys-fd-write"),
            fd_read: cfg!(feature = "wasip1-sys-fd-read"),
            fd_fdstat_get: cfg!(feature = "wasip1-sys-fd-fdstat-get"),
            fd_close: cfg!(feature = "wasip1-sys-fd-close"),
            clock_res_get: cfg!(feature = "wasip1-sys-clock-res-get"),
            clock_time_get: cfg!(feature = "wasip1-sys-clock-time-get"),
            poll_oneoff: cfg!(feature = "wasip1-sys-poll-oneoff"),
            random_get: cfg!(feature = "wasip1-sys-random-get"),
            proc_exit: cfg!(feature = "wasip1-sys-proc-exit"),
            proc_raise: cfg!(feature = "wasip1-sys-proc-raise"),
            sched_yield: cfg!(feature = "wasip1-sys-sched-yield"),
            path_minimal: cfg!(feature = "wasip1-sys-path-minimal"),
            path_full: cfg!(feature = "wasip1-sys-path-full"),
            network_object: cfg!(feature = "wasip1-sys-sock"),
        }
    }

    pub const fn supports(self, syscall: Wasip1Syscall) -> bool {
        match syscall {
            Wasip1Syscall::ArgsEnv => self.args_env,
            Wasip1Syscall::FdWrite => self.fd_write,
            Wasip1Syscall::FdRead => self.fd_read,
            Wasip1Syscall::FdFdstatGet => self.fd_fdstat_get,
            Wasip1Syscall::FdClose => self.fd_close,
            Wasip1Syscall::ClockResGet => self.clock_res_get,
            Wasip1Syscall::ClockTimeGet => self.clock_time_get,
            Wasip1Syscall::PollOneoff => self.poll_oneoff,
            Wasip1Syscall::RandomGet => self.random_get,
            Wasip1Syscall::ProcExit => self.proc_exit,
            Wasip1Syscall::ProcRaise => self.proc_raise,
            Wasip1Syscall::SchedYield => self.sched_yield,
            Wasip1Syscall::PathMinimal => self.path_minimal,
            Wasip1Syscall::PathFull => self.path_full,
            Wasip1Syscall::NetworkObject => self.network_object,
        }
    }

    pub const fn implemented_count(self) -> usize {
        self.args_env as usize
            + self.fd_write as usize
            + self.fd_read as usize
            + self.fd_fdstat_get as usize
            + self.fd_close as usize
            + self.clock_res_get as usize
            + self.clock_time_get as usize
            + self.poll_oneoff as usize
            + self.random_get as usize
            + self.proc_exit as usize
            + self.proc_raise as usize
            + self.sched_yield as usize
            + self.path_minimal as usize
            + self.path_full as usize
            + self.network_object as usize
    }

    pub const fn is_fullish(self) -> bool {
        self.args_env
            && self.fd_write
            && self.fd_read
            && self.fd_fdstat_get
            && self.fd_close
            && self.clock_res_get
            && self.clock_time_get
            && self.poll_oneoff
            && self.random_get
            && self.proc_exit
            && self.proc_raise
            && self.sched_yield
            && self.path_minimal
            && self.path_full
            && self.network_object
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wasip1ControlSubstrate {
    pub fd_view: bool,
    pub memory_lease: bool,
    pub errno: bool,
    pub import_validation: bool,
    pub unsupported_reject: bool,
    pub memory_grow_fence: bool,
}

impl Wasip1ControlSubstrate {
    pub const FULL: Self = Self {
        fd_view: true,
        memory_lease: true,
        errno: true,
        import_validation: true,
        unsupported_reject: true,
        memory_grow_fence: true,
    };

    pub const fn active() -> Self {
        Self {
            fd_view: cfg!(feature = "wasip1-ctrl-fd-view"),
            memory_lease: cfg!(feature = "wasip1-ctrl-memory-lease"),
            errno: cfg!(feature = "wasip1-ctrl-errno"),
            import_validation: cfg!(feature = "wasip1-ctrl-import-validation"),
            unsupported_reject: cfg!(feature = "wasip1-ctrl-unsupported-reject"),
            memory_grow_fence: cfg!(feature = "wasip1-ctrl-memory-grow-fence"),
        }
    }

    pub const fn is_complete_for_wasip1(self) -> bool {
        self.fd_view
            && self.memory_lease
            && self.errno
            && self.import_validation
            && self.unsupported_reject
            && self.memory_grow_fence
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeatureProfiles {
    pub rp2040_baker_min: bool,
    pub pico2w_swarm_min: bool,
    pub host_qemu_swarm: bool,
    pub host_linux_wasip1_full: bool,
}

impl FeatureProfiles {
    pub const fn active() -> Self {
        Self {
            rp2040_baker_min: cfg!(feature = "profile-rp2040-baker-min"),
            pico2w_swarm_min: cfg!(feature = "profile-pico2w-swarm-min"),
            host_qemu_swarm: cfg!(feature = "profile-host-qemu-swarm"),
            host_linux_wasip1_full: cfg!(feature = "profile-host-linux-wasip1-full"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeatureMatrix {
    pub profiles: FeatureProfiles,
    pub engine: WasmEngineProfile,
    pub wasip1_handlers: Wasip1HandlerSet,
    pub wasip1_control: Wasip1ControlSubstrate,
}

impl FeatureMatrix {
    pub const fn active() -> Self {
        Self {
            profiles: FeatureProfiles::active(),
            engine: WasmEngineProfile::active(),
            wasip1_handlers: Wasip1HandlerSet::active(),
            wasip1_control: Wasip1ControlSubstrate::active(),
        }
    }

    pub const fn can_claim_wasip1_profile(self) -> bool {
        self.engine.can_run_core_wasip1()
            && self.wasip1_control.is_complete_for_wasip1()
            && self.wasip1_handlers.implemented_count() > 0
    }

    pub const fn can_claim_full_ordinary_std(self) -> bool {
        self.engine.can_run_ordinary_wasip1_std()
            && self.wasip1_handlers.is_fullish()
            && self.wasip1_control.is_complete_for_wasip1()
    }
}

pub const ACTIVE_FEATURE_MATRIX: FeatureMatrix = FeatureMatrix::active();

#[cfg(test)]
mod tests {
    use super::{
        FeatureMatrix, Wasip1ControlSubstrate, Wasip1HandlerSet, Wasip1Syscall, WasmEngineProfile,
    };

    #[test]
    fn baker_min_profile_has_only_the_small_wasi_surface() {
        let handlers = Wasip1HandlerSet::BAKER_MIN;

        assert!(handlers.supports(Wasip1Syscall::FdWrite));
        assert!(handlers.supports(Wasip1Syscall::PollOneoff));
        assert!(handlers.supports(Wasip1Syscall::ProcExit));
        assert!(!handlers.supports(Wasip1Syscall::ProcRaise));
        assert!(!handlers.supports(Wasip1Syscall::FdRead));
        assert!(!handlers.supports(Wasip1Syscall::RandomGet));
        assert!(!handlers.is_fullish());
    }

    #[test]
    fn core_wasm_engine_profile_does_not_imply_wasi_syscalls() {
        let matrix = FeatureMatrix {
            profiles: Default::default(),
            engine: WasmEngineProfile::Core,
            wasip1_handlers: Wasip1HandlerSet::EMPTY,
            wasip1_control: Wasip1ControlSubstrate::FULL,
        };

        assert!(matrix.engine.can_run_core_wasip1());
        assert!(!matrix.wasip1_handlers.supports(Wasip1Syscall::ProcExit));
        assert!(!matrix.wasip1_handlers.supports(Wasip1Syscall::FdWrite));
        assert!(!matrix.can_claim_wasip1_profile());
    }

    #[test]
    fn full_profile_requires_engine_handlers_and_common_control_substrate() {
        let matrix = FeatureMatrix {
            profiles: Default::default(),
            engine: WasmEngineProfile::Wasip1Full,
            wasip1_handlers: Wasip1HandlerSet::FULL,
            wasip1_control: Wasip1ControlSubstrate::FULL,
        };

        assert!(matrix.can_claim_wasip1_profile());
        assert!(matrix.can_claim_full_ordinary_std());
    }
}
