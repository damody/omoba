use omoba_core::runtime::{run_td_autoplay_1_to_100, TdAutoplayRunConfig};

#[test]
fn layered_td_coarse_autoplay_completes_rounds_1_to_100() {
    let scripts =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/target/release");
    let config = TdAutoplayRunConfig::coarse_1_to_100(scripts);
    let report = run_td_autoplay_1_to_100(&config).expect("1-100 autoplay run");
    assert_eq!(report.round_end_ticks.len(), 100);
    assert!(report
        .round_end_ticks
        .last()
        .is_some_and(|tick| *tick <= report.ticks));
    assert!(report.lives > 0);
    assert_ne!(report.ledger_digest, 0);
    assert_ne!(report.state_hash, 0);
    assert!(report.ticks_per_wall_second.is_finite());
}

#[test]
fn autoplay_failure_report_contains_actionable_context() {
    let scripts =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/target/release");
    let mut config = TdAutoplayRunConfig::coarse_1_to_100(scripts);
    config.max_ticks = 0;
    let error = run_td_autoplay_1_to_100(&config).expect_err("zero tick budget must fail");
    assert!(error.contains("maximum tick budget exceeded"));

    let report_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/td-autoplay/failure.txt");
    let report = std::fs::read_to_string(&report_path).expect("failure report is readable");
    for required in [
        "reason=",
        "watchdog_state=",
        "seed=",
        "profile=",
        "round=",
        "tick=",
        "lives=",
        "cash=",
        "towers=",
        "remaining_enemies=",
        "entity_peak=",
        "ledger_totals=",
        "ledger_digest=",
        "state_hash=",
        "recent_outcomes=",
        "recent_rejected_inputs=",
    ] {
        assert!(report.contains(required), "missing report field {required}");
    }
    let _ = std::fs::remove_file(report_path);
}
