use omoba_core::runtime::{run_td_autoplay_1_to_100, TdAutoplayRunConfig};

fn main() -> Result<(), String> {
    let scripts_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../scripts/target/release".to_string());
    let config = TdAutoplayRunConfig::coarse_1_to_100(scripts_dir);
    let first = run_td_autoplay_1_to_100(&config)?;
    let second = run_td_autoplay_1_to_100(&config)?;
    if first.state_hash != second.state_hash
        || first.ledger_digest != second.ledger_digest
        || first.round_end_ticks != second.round_end_ticks
    {
        return Err(format!(
            "deterministic replay mismatch:\nfirst={}\nsecond={}",
            first.compact_summary(),
            second.compact_summary()
        ));
    }
    println!("{}", first.compact_summary());
    Ok(())
}
