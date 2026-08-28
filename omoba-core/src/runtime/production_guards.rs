#[test]
fn production_selective_consumers_cannot_use_noop_stepper() {
    let observer = include_str!("observer_validation.rs");
    let omfx = include_str!("../../../omfx/game/src/sim_runner.rs");
    assert!(!observer.contains("NoopDisclosedWorldStepper"));
    assert!(!omfx.contains("NoopDisclosedWorldStepper"));
}

#[test]
fn authoritative_runtime_consumes_the_shared_phase_table() {
    let backend = include_str!("../../../omb/src/state/core.rs");
    assert!(backend.contains("run_deterministic_gameplay_phases"));
    assert!(!backend.contains("const DETERMINISTIC_GAMEPLAY_PHASES"));
    let omfx = include_str!("../../../omfx/game/src/sim_runner.rs");
    assert!(omfx.contains("run_deterministic_gameplay_phases"));
    assert!(!omfx.contains("const DETERMINISTIC_GAMEPLAY_PHASES"));
    let expected = super::DETERMINISTIC_GAMEPLAY_PHASES;
    assert_eq!(
        expected.first(),
        Some(&super::DeterministicGameplayPhase::Dispatcher)
    );
    assert_eq!(
        expected.last(),
        Some(&super::DeterministicGameplayPhase::PostScriptOutcomes)
    );
    assert_eq!(expected.len(), 18);
}

#[test]
fn steady_state_projection_does_not_call_visible_demo_repairs() {
    let projector = include_str!("team_projector.rs");
    let production = projector
        .split("impl TeamViewProjector")
        .next()
        .unwrap_or(projector);
    assert!(!production.contains("enqueue_visible_demo_repairs("));
}

#[test]
fn fixed_observer_team_contract_is_exactly_two_teams() {
    assert_eq!(super::SUPPORTED_REPLICA_TEAMS, [1, 2]);
}

#[test]
fn broadcaster_fans_out_the_same_encoded_payload_to_network_and_observer() {
    let transport = include_str!("../../../omb/src/transport/kcp_transport.rs");
    assert!(transport.contains("bytes: Arc::clone(&encoded)"));
    assert!(transport.contains("build_framed_bytes(TAG_TEAM_TICK_FRAME_V2, &encoded)"));
    assert!(transport.contains("observer_tap_broadcast.try_frame"));
    assert!(transport.contains("Arc::clone(&encoded)"));
}

#[test]
fn world_maintain_is_limited_to_the_two_outcome_boundaries() {
    for source in [
        include_str!("../../../omb/src/state/core.rs"),
        include_str!("../../../omfx/game/src/sim_runner.rs"),
        include_str!("filtered_specs.rs"),
    ] {
        assert!(source.contains("matches!(phase, P::PreScriptOutcomes | P::PostScriptOutcomes)"));
    }
}

#[test]
fn gameplay_rng_does_not_seed_by_entity_system_or_action() {
    for source in [
        include_str!("native/tick/tower_tick.rs"),
        include_str!("native/tick/hero_tick.rs"),
        include_str!("native/tick/damage_tick.rs"),
        include_str!("native/game_processor.rs"),
        include_str!("native/scripting/parallel_world_adapter.rs"),
    ] {
        assert!(!source.contains("from_master_entity("));
        assert!(!source.contains("from_master_ordinal("));
    }
}

#[test]
fn secure_replica_allowlists_have_one_production_source() {
    for source in [
        include_str!("team_projector.rs"),
        include_str!("observer_validation.rs"),
        include_str!("../../../omfx/game/src/sim_runner.rs"),
    ] {
        assert!(source.contains("secure_replica_component_allowlist"));
        assert!(!source.contains("component_allowlist: BTreeSet::from"));
        assert!(!source.contains("resource_allowlist: BTreeSet::from"));
    }
}

#[test]
fn renderer_only_ipc_module_has_no_gameplay_or_server_transport_owner() {
    let source = include_str!("../../../omfx/game/src/presentation_client.rs");
    for forbidden in [
        "specs::",
        "SpecsDisclosedWorldStepper",
        "SelectiveReplicaRuntime",
        "KcpClient",
        "base_content.dll",
        "ScriptRegistry",
    ] {
        assert!(
            !source.contains(forbidden),
            "renderer-only module contains {forbidden}"
        );
    }
}

#[test]
fn fog_demo_contract_assets_are_explicit() {
    let map = include_str!("../../../scripts/lua_data/FOG_2TEAM_DEMO/map.lua");
    for required in [
        "Rows = 10",
        "Columns = 10",
        "VisionRadius = 700.0",
        "LastKnownIndexes",
        "VisionTrees",
        "VisionOccluderPolygons",
        "PatrolIndexes",
    ] {
        assert!(map.contains(required), "fog demo missing {required}");
    }
    let initialization = include_str!("native/initialization.rs");
    assert!(initialization.contains("assert_eq!(team_counts, [34, 33, 33])"));
    assert!(initialization.contains("grid=100 heroes=2"));
}

#[test]
fn evidence_tools_fail_closed_and_never_kill_by_image_name() {
    let compare = include_str!("../../../scripts/compare_fog_evidence.lua");
    assert!(compare.contains("UNVERIFIED"));
    assert!(compare.contains("opponent-sentinel-absence"));
    let launcher = include_str!("../../../run_2player.bat");
    assert!(!launcher.to_ascii_lowercase().contains("taskkill"));
    let process = include_str!("../../../tools/lua/lib/process.lua");
    assert!(process.contains("assert_identity"));
}

#[test]
fn netem_proxy_is_transport_only_and_launcher_termination_is_pid_scoped() {
    let manifest = include_str!("../../../omoba-netem-proxy/Cargo.toml");
    let source = [
        include_str!("../../../omoba-netem-proxy/src/lib.rs"),
        include_str!("../../../omoba-netem-proxy/src/route.rs"),
        include_str!("../../../omoba-netem-proxy/src/runtime.rs"),
    ]
    .join("\n");
    for forbidden in ["specs", "fyrox", "base_content", "omoba-core", "script-abi"] {
        assert!(
            !manifest.to_ascii_lowercase().contains(forbidden),
            "netem manifest contains gameplay dependency {forbidden}"
        );
        assert!(
            !source.to_ascii_lowercase().contains(forbidden),
            "netem source contains gameplay dependency {forbidden}"
        );
    }
    let stop = include_str!("../../../scripts/stop_netem_proxy.lua");
    assert!(!stop.to_ascii_lowercase().contains("taskkill"));
    assert!(stop.contains("assert_identity"));
    assert!(stop.contains("expected-exe"));
    let reveal_after_hide = include_str!(
        "../../../openspec/changes/simulate-client-rtt-delay/fixtures/reveal-after-hide.json"
    );
    let reveal_after_forget = include_str!(
        "../../../openspec/changes/simulate-client-rtt-delay/fixtures/reveal-after-forget.json"
    );
    assert!(reveal_after_hide.contains("\"arrival_order\": [2, 1]"));
    assert!(reveal_after_hide.contains("\"expected_final_state\": \"hidden\""));
    assert!(reveal_after_forget.contains("\"expected_final_state\": \"retired\""));
}
