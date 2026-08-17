use std::time::Duration;

use omoba_core::runtime::{
    run_td_autoplay_1_to_100, run_td_autoplay_1_to_100_observed,
    TdAutoplayObservedOutcome, TdAutoplayObserverControl, TdAutoplayRunConfig,
    TdAutoplayRunStatus,
};

#[test]
fn layered_td_coarse_autoplay_completes_rounds_1_to_100() {
    let scripts =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/target/release");
    let config = TdAutoplayRunConfig::coarse_1_to_100(scripts);
    let headless = run_td_autoplay_1_to_100(&config).expect("headless 1-100 autoplay run");
    assert_eq!(headless.round_end_ticks.len(), 100);
    assert!(headless
        .round_end_ticks
        .last()
        .is_some_and(|tick| *tick <= headless.ticks));
    assert!(headless.lives > 0);
    assert_ne!(headless.ledger_digest, 0);
    assert_ne!(headless.state_hash, 0);
    assert!(headless.ticks_per_wall_second.is_finite());

    let mut statuses = Vec::new();
    let mut final_frame = None;
    let observed = run_td_autoplay_1_to_100_observed(
        &config,
        Duration::from_secs(60 * 60),
        |frame| {
            statuses.push(frame.status);
            final_frame = Some((
                frame.round,
                frame.total_rounds,
                frame.cash,
                frame.lives,
                frame.tick,
            ));
            TdAutoplayObserverControl::Continue
        },
    )
    .expect("observed 1-100 autoplay run");
    let TdAutoplayObservedOutcome::Completed(observed) = observed else {
        panic!("observed autoplay unexpectedly cancelled");
    };
    assert_eq!(observed.round_end_ticks, headless.round_end_ticks);
    assert_eq!(observed.ledger_digest, headless.ledger_digest);
    assert_eq!(observed.state_hash, headless.state_hash);
    assert_eq!(observed.cash, headless.cash);
    assert_eq!(observed.lives, headless.lives);
    assert_eq!(statuses.first(), Some(&TdAutoplayRunStatus::Running));
    assert_eq!(statuses.last(), Some(&TdAutoplayRunStatus::Completed));
    assert_eq!(
        final_frame,
        Some((100, 100, observed.cash, observed.lives, observed.ticks))
    );
}

#[test]
fn observed_autoplay_can_cancel_before_the_first_tick() {
    let scripts =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/target/release");
    let config = TdAutoplayRunConfig::coarse_1_to_100(scripts);
    let mut statuses = Vec::new();
    let outcome = run_td_autoplay_1_to_100_observed(&config, Duration::from_millis(100), |frame| {
        statuses.push(frame.status);
        if frame.status == TdAutoplayRunStatus::Running {
            TdAutoplayObserverControl::Cancel
        } else {
            TdAutoplayObserverControl::Continue
        }
    })
    .expect("observed autoplay cancellation");
    assert_eq!(outcome, TdAutoplayObservedOutcome::Cancelled);
    assert_eq!(
        statuses,
        vec![TdAutoplayRunStatus::Running, TdAutoplayRunStatus::Cancelled]
    );
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
