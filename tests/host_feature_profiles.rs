use hibana_pico::kernel::features::{
    FeatureMatrix, WASIP1_PREVIEW1_IMPORT_COVERAGE, WASIP1_PREVIEW1_IMPORTS,
    Wasip1ControlSubstrate, Wasip1HandlerSet, Wasip1ImportDisposition,
    Wasip1ImportEffectiveDisposition, Wasip1ImportName, Wasip1Syscall, WasmEngineProfile,
};

fn cargo_toml() -> &'static str {
    include_str!("../Cargo.toml")
}

#[test]
fn cargo_features_define_small_pico_and_full_host_profiles() {
    let cargo = cargo_toml();

    for feature in [
        "profile-rp2040-pico-min",
        "profile-rp2040-picow-swarm-min",
        "profile-rp2350-pico2w-swarm-min",
        "profile-host-qemu-swarm",
        "profile-host-linux-wasip1-full",
        "wasm-engine-core",
        "wasm-engine-wasip1-full",
        "wasip1-sys-full",
        "wasip1-ctrl-common",
        "wasip1-ledger-pico-min",
        "wasip1-ledger-embedded-std",
        "wasip1-ledger-host-full",
    ] {
        assert!(cargo.contains(feature), "Cargo.toml is missing {feature}");
    }

    assert!(cargo.contains("\"wasm-engine-core\""));
    assert!(cargo.contains("\"wasip1-sys-fd-write\""));
    assert!(cargo.contains("\"wasip1-sys-poll-oneoff\""));
    assert!(cargo.contains("\"wasip1-sys-proc-exit\""));
    assert!(cargo.contains("\"wasip1-sys-proc-raise\""));
    assert!(cargo.contains("\"wasm-engine-wasip1-full\""));
    assert!(cargo.contains("\"wasip1-sys-full\""));
    assert!(cargo.contains("\"wasip1-ledger-pico-min\""));
}

#[test]
fn feature_control_matrix_keeps_pico_small_and_host_full_as_separate_axes() {
    let pico = FeatureMatrix {
        profiles: Default::default(),
        engine: WasmEngineProfile::Core,
        wasip1_handlers: Wasip1HandlerSet::PICO_MIN,
        wasip1_control: Wasip1ControlSubstrate::FULL,
    };
    let host = FeatureMatrix {
        profiles: Default::default(),
        engine: WasmEngineProfile::Wasip1Full,
        wasip1_handlers: Wasip1HandlerSet::FULL,
        wasip1_control: Wasip1ControlSubstrate::FULL,
    };

    assert!(pico.can_claim_wasip1_profile());
    assert!(!pico.can_claim_full_ordinary_std());
    assert!(pico.wasip1_handlers.supports(Wasip1Syscall::FdWrite));
    assert!(pico.wasip1_handlers.supports(Wasip1Syscall::PollOneoff));
    assert!(!pico.wasip1_handlers.supports(Wasip1Syscall::FdRead));

    assert!(host.can_claim_wasip1_profile());
    assert!(host.can_claim_full_ordinary_std());
    assert!(host.wasip1_handlers.supports(Wasip1Syscall::FdRead));
    assert!(host.wasip1_handlers.supports(Wasip1Syscall::RandomGet));
    assert!(host.wasip1_handlers.supports(Wasip1Syscall::ProcRaise));
}

#[test]
fn wasi_p1_import_coverage_table_is_the_source_of_truth_for_profiles() {
    assert_eq!(WASIP1_PREVIEW1_IMPORT_COVERAGE.len(), 46);
    assert_eq!(
        WASIP1_PREVIEW1_IMPORT_COVERAGE.len(),
        WASIP1_PREVIEW1_IMPORTS.len()
    );
    for (coverage, import) in WASIP1_PREVIEW1_IMPORT_COVERAGE
        .iter()
        .zip(WASIP1_PREVIEW1_IMPORTS.iter())
    {
        assert_eq!(coverage.kind, *import);
        assert_eq!(coverage.import, import.name());
        assert_eq!(coverage.syscall, import.syscall());
        assert_eq!(coverage.disposition, import.disposition());
    }
    assert_eq!(
        Wasip1ImportName::from_bytes(b"fd_write"),
        Some(Wasip1ImportName::FdWrite)
    );

    for required in [
        "fd_write",
        "fd_read",
        "path_open",
        "fd_pwrite",
        "sock_send",
        "sock_recv",
        "sock_accept",
        "proc_raise",
    ] {
        assert!(
            WASIP1_PREVIEW1_IMPORT_COVERAGE
                .iter()
                .any(|entry| entry.import == required),
            "coverage table missing {required}"
        );
    }

    let full = Wasip1HandlerSet::FULL;
    assert!(
        WASIP1_PREVIEW1_IMPORT_COVERAGE
            .iter()
            .all(|entry| entry.effective(full)
                != Wasip1ImportEffectiveDisposition::UnsupportedByProfile),
        "full profile must classify every Preview 1 import"
    );
    assert!(
        WASIP1_PREVIEW1_IMPORT_COVERAGE
            .iter()
            .any(|entry| entry.disposition == Wasip1ImportDisposition::TypedEnosys),
        "coverage table must make ENOSYS imports explicit"
    );
    assert!(
        WASIP1_PREVIEW1_IMPORT_COVERAGE
            .iter()
            .any(|entry| entry.disposition == Wasip1ImportDisposition::TypedReject),
        "coverage table must make fail-closed reject imports explicit"
    );

    let pico = Wasip1HandlerSet::PICO_MIN;
    assert_eq!(
        WASIP1_PREVIEW1_IMPORT_COVERAGE
            .iter()
            .find(|entry| entry.import == "fd_write")
            .expect("fd_write coverage")
            .effective(pico),
        Wasip1ImportEffectiveDisposition::Supported
    );
    assert_eq!(
        WASIP1_PREVIEW1_IMPORT_COVERAGE
            .iter()
            .find(|entry| entry.import == "path_open")
            .expect("path_open coverage")
            .effective(pico),
        Wasip1ImportEffectiveDisposition::UnsupportedByProfile
    );
    assert_eq!(
        WASIP1_PREVIEW1_IMPORT_COVERAGE
            .iter()
            .find(|entry| entry.import == "sock_send")
            .expect("sock_send coverage")
            .effective(pico),
        Wasip1ImportEffectiveDisposition::UnsupportedByProfile
    );
}

#[test]
fn wasi_p1_import_names_are_not_redeclared_as_manual_byte_tables() {
    for (path, source) in [
        ("src/kernel/wasi.rs", include_str!("../src/kernel/wasi.rs")),
        (
            "src/kernel/engine/wasm.rs",
            include_str!("../src/kernel/engine/wasm.rs"),
        ),
    ] {
        for import in WASIP1_PREVIEW1_IMPORTS {
            let forbidden = format!("= b\"{}\"", import.name());
            assert!(
                !source.contains(&forbidden),
                "{path} redeclares WASI P1 import name {forbidden}; use Wasip1ImportName"
            );
        }
    }
}

#[test]
fn choreography_sources_do_not_use_feature_cfg_as_protocol_authority() {
    const CHOREOGRAPHY_SOURCES: &[(&str, &str)] = &[
        (
            "src/choreography/protocol.rs",
            include_str!("../src/choreography/protocol.rs"),
        ),
        (
            "src/choreography/local.rs",
            include_str!("../src/choreography/local.rs"),
        ),
        (
            "src/choreography/swarm.rs",
            include_str!("../src/choreography/swarm.rs"),
        ),
    ];

    for (path, source) in CHOREOGRAPHY_SOURCES {
        assert!(
            !source.contains("feature ="),
            "{path} must not gate protocol shape on Cargo features"
        );
    }
}

#[test]
fn ordinary_std_corpus_is_full_profile_engine_coverage_not_choreography_policy() {
    let cargo = cargo_toml();
    assert!(cargo.contains("wasm-engine-wasip1-full"));
    assert!(cargo.contains("wasip1-sys-full"));

    let smoke_manifest = include_str!("../apps/wasip1/wasip1-smoke-apps/Cargo.toml");
    assert!(smoke_manifest.contains("wasip1-std-core-coverage"));

    let source =
        include_str!("../apps/wasip1/wasip1-smoke-apps/src/bin/wasip1-std-core-coverage.rs");
    for needle in [
        "Vec::",
        "String::",
        "match ",
        "memory_grow",
        "sqrt",
        "File::from_raw_fd",
    ] {
        assert!(source.contains(needle), "coverage app is missing {needle}");
    }
    assert!(
        !source.contains("#![no_main]") && !source.contains("__main_void"),
        "coverage app must remain an ordinary Rust std fn main artifact"
    );
}

#[test]
#[cfg(feature = "profile-host-linux-wasip1-full")]
fn active_host_linux_full_profile_claims_full_ordinary_std_capacity() {
    use hibana_pico::kernel::features::ACTIVE_FEATURE_MATRIX;

    assert!(ACTIVE_FEATURE_MATRIX.profiles.host_linux_wasip1_full);
    assert!(ACTIVE_FEATURE_MATRIX.can_claim_full_ordinary_std());
}

#[test]
#[cfg(feature = "profile-rp2040-pico-min")]
fn active_rp2040_pico_profile_is_small_not_full_std() {
    use hibana_pico::kernel::features::ACTIVE_FEATURE_MATRIX;

    assert!(ACTIVE_FEATURE_MATRIX.profiles.rp2040_pico_min);
    assert!(ACTIVE_FEATURE_MATRIX.can_claim_wasip1_profile());
    assert!(!ACTIVE_FEATURE_MATRIX.can_claim_full_ordinary_std());
}

#[test]
#[cfg(feature = "profile-rp2040-picow-swarm-min")]
fn active_rp2040_picow_profile_is_wireless_capacity_not_full_std() {
    use hibana_pico::kernel::features::ACTIVE_FEATURE_MATRIX;

    assert!(ACTIVE_FEATURE_MATRIX.profiles.rp2040_picow_swarm_min);
    assert!(ACTIVE_FEATURE_MATRIX.can_claim_wasip1_profile());
    assert!(!ACTIVE_FEATURE_MATRIX.can_claim_full_ordinary_std());
}

#[test]
#[cfg(feature = "profile-rp2350-pico2w-swarm-min")]
fn active_rp2350_pico2w_profile_is_wireless_capacity_not_full_std() {
    use hibana_pico::kernel::features::ACTIVE_FEATURE_MATRIX;

    assert!(ACTIVE_FEATURE_MATRIX.profiles.rp2350_pico2w_swarm_min);
    assert!(ACTIVE_FEATURE_MATRIX.can_claim_wasip1_profile());
    assert!(!ACTIVE_FEATURE_MATRIX.can_claim_full_ordinary_std());
}
