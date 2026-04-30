use hibana_pico::kernel::metrics::{NO_P2_METRICS, SINGLE_NODE_METRICS, SWARM_METRICS};

#[test]
fn plan_pico_single_node_measurements_stay_bounded() {
    let metrics = SINGLE_NODE_METRICS;

    assert_eq!(metrics.sio_role_capacity, 4);
    assert_eq!(metrics.sio_queue_capacity, 8);
    assert_eq!(metrics.sio_frame_payload_capacity, 96);
    assert!(metrics.sio_frame_size <= 128);
    assert!(metrics.syscall_request_size <= 64);
    assert!(metrics.syscall_response_size <= 64);
    assert_eq!(metrics.syscall_buffer_capacity, 30);
    assert!(metrics.timer_table_size <= 192);
    assert!(metrics.interrupt_resolver_size <= 320);
    assert!(metrics.interrupt_resolver_rejection_telemetry_size <= 16);
    assert!(metrics.budget_controller_size <= 32);
    assert!(metrics.memory_lease_table_size <= 128);
    assert!(metrics.memory_lease_rejection_telemetry_size <= 16);
    assert!(metrics.pico_fd_view_size <= 256);
    assert!(metrics.pico_fd_rejection_telemetry_size <= 16);
    assert!(metrics.app_stream_table_size <= 128);
    assert!(metrics.app_lease_table_size <= 128);
    assert!(metrics.endpoint_size <= 32);
    assert!(metrics.role_program_size <= 16);
    assert!(metrics.static_arg_env_size <= 256);
    assert!(metrics.image_slot_table_size <= 2048);
    assert!(metrics.management_rejection_telemetry_size <= 16);
}

#[test]
fn plan_pico_swarm_measurements_stay_bounded() {
    let metrics = SWARM_METRICS;

    assert_eq!(metrics.wifi_frame_header_len, 28);
    assert_eq!(metrics.wifi_frame_payload_capacity, 96);
    assert_eq!(metrics.wifi_auth_tag_len, 4);
    assert_eq!(metrics.wifi_frame_max_wire_len, 128);
    assert!(metrics.wifi_frame_size <= 128);
    assert_eq!(metrics.fragmentation_header_len, 6);
    assert_eq!(metrics.fragmentation_chunk_capacity, 90);
    assert!(metrics.fragmentation_buffer_size <= 320);
    assert_eq!(metrics.qemu_cyw_max_roles, 6);
    assert!(metrics.qemu_cyw_transport_size <= 64);
    assert!(metrics.host_swarm_medium_size <= 2048);
    assert!(metrics.host_swarm_role_transport_size <= 128);
    assert!(metrics.neighbor_table_size <= 64);
    assert!(metrics.remote_object_table_size <= 256);
    assert!(metrics.remote_object_rejection_telemetry_size <= 16);
    assert!(metrics.replay_window_size <= 8);
    assert!(metrics.swarm_drop_telemetry_size <= 16);
    assert_eq!(metrics.wifi_ping_pong_nodes, 2);
    assert_eq!(metrics.wifi_ping_pong_messages, 2);
    assert_eq!(metrics.remote_fd_read_messages, 2);
    assert_eq!(metrics.remote_actuator_command_messages, 2);
    assert_eq!(metrics.packet_loss_retry_frames, 1);
    assert_eq!(metrics.provisioning_join_messages, 4);
    assert_eq!(metrics.leave_revoke_messages, 4);
    assert_eq!(metrics.qemu_swarm_default_nodes, 6);
    assert_eq!(metrics.qemu_swarm_default_sensor_nodes, 5);
    assert_eq!(metrics.qemu_swarm_sample_count, 5);
    assert_eq!(metrics.qemu_swarm_wasip1_fd_write_count, 5);
    assert_eq!(metrics.qemu_swarm_aggregate_ack_count, 5);
    assert_eq!(metrics.qemu_swarm_base_sample_value, 0x0000_a5a5);
    assert_eq!(metrics.qemu_swarm_default_aggregate, 0x0003_3c43);
}

#[test]
fn plan_pico_cyw43439_firmware_artifact_is_picosdk_sourced_and_disassembled() {
    let firmware = std::fs::read("firmware/cyw43/w43439A0_7_95_49_00_firmware.bin")
        .expect("run scripts/extract_cyw43_firmware.py before plan gate");
    let clm = std::fs::read("firmware/cyw43/w43439A0_7_95_49_00_clm.bin")
        .expect("run scripts/extract_cyw43_firmware.py before plan gate");
    let manifest = std::fs::read_to_string("firmware/cyw43/w43439A0_7_95_49_00.manifest.json")
        .expect("run scripts/extract_cyw43_firmware.py before plan gate");
    let disasm = std::fs::read_to_string(
        "firmware/cyw43/w43439A0_7_95_49_00_firmware.thumb.disasm.head.txt",
    )
    .expect("run scripts/disassemble_cyw43_firmware.sh before plan gate");

    assert_eq!(firmware.len(), 224_190);
    assert_eq!(clm.len(), 984);
    assert!(manifest.contains("a1438dff1d38bd9c65dbd693f0e5db4b9ae91779"));
    assert!(manifest.contains("dd7568229f3bf7a37737b9e1ef250c26efe75b23"));
    assert!(manifest.contains("\"fnv1a32\": \"0xfa231a9f\""));
    assert!(manifest.contains("\"fnv1a32\": \"0x5178f94d\""));
    assert!(disasm.contains("Disassembly of section .text"));
    assert!(disasm.contains("_binary_firmware_cyw43_w43439A0_7_95_49_00_firmware_bin_start"));
}

#[test]
fn plan_pico_no_p2_measurements_stay_absent() {
    let metrics = NO_P2_METRICS;

    assert_eq!(metrics.wasip1_full_subset_import_count, 46);
    assert_eq!(metrics.wasip1_static_args_capacity, 4);
    assert_eq!(metrics.wasip1_static_env_capacity, 4);
    assert_eq!(metrics.wasip1_static_arg_bytes_capacity, 64);
    assert_eq!(metrics.wasip1_static_env_bytes_capacity, 128);
    assert!(metrics.network_object_table_size <= 256);
    assert!(metrics.network_object_rejection_telemetry_size <= 16);
    assert_eq!(metrics.network_datagram_payload_capacity, 48);
    assert_eq!(metrics.network_stream_payload_capacity, 48);
    assert_eq!(metrics.component_model_loader_bytes, 0);
    assert_eq!(metrics.wit_runtime_table_bytes, 0);
    assert_eq!(metrics.p2_resource_table_bytes, 0);
}

#[test]
fn plan_pico_source_tree_keeps_no_p2_runtime_surface() {
    const NEEDLES: &[&str] = &[
        "wasi:cli",
        "wasi:clocks",
        "wasi:filesystem",
        "wasi:http",
        "wasi:io",
        "wasi:random",
        "wasi:sockets",
        "wasi/",
        "wasm32-wasip2",
        "wasip2",
        "wasi_snapshot_preview2",
        "preview2",
        "wit-bindgen",
        "wit_component",
        "component-model",
    ];
    const SOURCES: &[(&str, &str)] = &[
        ("Cargo.toml", include_str!("../Cargo.toml")),
        ("README.md", include_str!("../README.md")),
        ("src/kernel/app.rs", include_str!("../src/kernel/app.rs")),
        (
            "src/substrate/host_queue.rs",
            include_str!("../src/substrate/host_queue.rs"),
        ),
        (
            "src/kernel/budget.rs",
            include_str!("../src/kernel/budget.rs"),
        ),
        (
            "src/machine/rp2350/cyw43439.rs",
            include_str!("../src/machine/rp2350/cyw43439.rs"),
        ),
        (
            "src/kernel/device/gpio.rs",
            include_str!("../src/kernel/device/gpio.rs"),
        ),
        ("src/lib.rs", include_str!("../src/lib.rs")),
        (
            "src/kernel/metrics.rs",
            include_str!("../src/kernel/metrics.rs"),
        ),
        ("src/kernel/mgmt.rs", include_str!("../src/kernel/mgmt.rs")),
        (
            "src/kernel/network.rs",
            include_str!("../src/kernel/network.rs"),
        ),
        (
            "src/kernel/policy.rs",
            include_str!("../src/kernel/policy.rs"),
        ),
        (
            "src/kernel/remote.rs",
            include_str!("../src/kernel/remote.rs"),
        ),
        (
            "src/kernel/resolver.rs",
            include_str!("../src/kernel/resolver.rs"),
        ),
        (
            "src/kernel/guest_ledger.rs",
            include_str!("../src/kernel/guest_ledger.rs"),
        ),
        (
            "src/kernel/swarm.rs",
            include_str!("../src/kernel/swarm.rs"),
        ),
        (
            "src/choreography/protocol.rs",
            include_str!("../src/choreography/protocol.rs"),
        ),
        (
            "src/kernel/device/timer.rs",
            include_str!("../src/kernel/device/timer.rs"),
        ),
        (
            "src/substrate/transport.rs",
            include_str!("../src/substrate/transport.rs"),
        ),
        ("src/kernel/wasi.rs", include_str!("../src/kernel/wasi.rs")),
        (
            "src/kernel/engine/wasm.rs",
            include_str!("../src/kernel/engine/wasm.rs"),
        ),
    ];

    for (path, source) in SOURCES {
        for needle in NEEDLES {
            assert!(
                !source.contains(needle),
                "{path} contains forbidden No-P2 runtime surface marker {needle:?}"
            );
        }
    }
}

#[test]
fn plan_pico_source_tree_keeps_no_bridge_runtime_surface() {
    const NEEDLES: &[&str] = &[
        "PicoBridge",
        "BridgeAdvance",
        "typed phase bridge",
        "bridge_state_size",
        "host_bridge",
        "send_packet_to_remote",
        "fd.is_remote(",
    ];
    const SOURCES: &[(&str, &str)] = &[
        ("src/kernel/app.rs", include_str!("../src/kernel/app.rs")),
        (
            "src/substrate/host_queue.rs",
            include_str!("../src/substrate/host_queue.rs"),
        ),
        (
            "src/kernel/budget.rs",
            include_str!("../src/kernel/budget.rs"),
        ),
        (
            "src/machine/rp2350/cyw43439.rs",
            include_str!("../src/machine/rp2350/cyw43439.rs"),
        ),
        (
            "src/kernel/device/gpio.rs",
            include_str!("../src/kernel/device/gpio.rs"),
        ),
        ("src/lib.rs", include_str!("../src/lib.rs")),
        (
            "src/kernel/metrics.rs",
            include_str!("../src/kernel/metrics.rs"),
        ),
        ("src/kernel/mgmt.rs", include_str!("../src/kernel/mgmt.rs")),
        (
            "src/kernel/network.rs",
            include_str!("../src/kernel/network.rs"),
        ),
        (
            "src/kernel/policy.rs",
            include_str!("../src/kernel/policy.rs"),
        ),
        (
            "src/kernel/remote.rs",
            include_str!("../src/kernel/remote.rs"),
        ),
        (
            "src/kernel/resolver.rs",
            include_str!("../src/kernel/resolver.rs"),
        ),
        (
            "src/kernel/guest_ledger.rs",
            include_str!("../src/kernel/guest_ledger.rs"),
        ),
        (
            "src/kernel/swarm.rs",
            include_str!("../src/kernel/swarm.rs"),
        ),
        (
            "src/choreography/protocol.rs",
            include_str!("../src/choreography/protocol.rs"),
        ),
        (
            "src/kernel/device/timer.rs",
            include_str!("../src/kernel/device/timer.rs"),
        ),
        (
            "src/substrate/transport.rs",
            include_str!("../src/substrate/transport.rs"),
        ),
        ("src/kernel/wasi.rs", include_str!("../src/kernel/wasi.rs")),
        (
            "src/kernel/engine/wasm.rs",
            include_str!("../src/kernel/engine/wasm.rs"),
        ),
    ];

    for (path, source) in SOURCES {
        for needle in NEEDLES {
            assert!(
                !source.contains(needle),
                "{path} contains forbidden bridge/relay runtime surface marker {needle:?}"
            );
        }
    }
}

#[test]
fn plan_pico_source_tree_keeps_removed_compatibility_names_out() {
    const NEEDLES: &[&str] = &[
        "Authorizer",
        "NetworkFd",
        "ListenerFd",
        "CompatibilityTier",
        "PicoFdResource",
        "PicoFdTable",
        "PicoFdEntry",
        "grant_routed",
        "remote_capability_table_size",
        "remote_capability_rejection_telemetry_size",
    ];
    const SOURCES: &[(&str, &str)] = &[
        (
            "src/kernel/features.rs",
            include_str!("../src/kernel/features.rs"),
        ),
        (
            "src/kernel/metrics.rs",
            include_str!("../src/kernel/metrics.rs"),
        ),
        ("src/kernel/mgmt.rs", include_str!("../src/kernel/mgmt.rs")),
        (
            "src/kernel/network.rs",
            include_str!("../src/kernel/network.rs"),
        ),
        (
            "src/kernel/remote.rs",
            include_str!("../src/kernel/remote.rs"),
        ),
        (
            "src/kernel/guest_ledger.rs",
            include_str!("../src/kernel/guest_ledger.rs"),
        ),
        ("src/kernel/wasi.rs", include_str!("../src/kernel/wasi.rs")),
        (
            "src/kernel/engine/wasip1_host.rs",
            include_str!("../src/kernel/engine/wasip1_host.rs"),
        ),
    ];

    for (path, source) in SOURCES {
        for needle in NEEDLES {
            assert!(
                !source.contains(needle),
                "{path} contains removed compatibility/control-bypass marker {needle:?}"
            );
        }
    }
}

#[test]
fn plan_pico_document_keeps_wasi_import_trampoline_name_no_bridge() {
    let plan = include_str!("../plan.md");

    assert!(
        !plan.contains("PicoWasiBridge"),
        "plan.md must not describe the WASI import trampoline as a bridge owner"
    );
    assert!(
        plan.contains("PicoWasiImportTrampoline"),
        "plan.md should name the WASI import owner as PicoWasiImportTrampoline"
    );
}
