//! Offline status of a local run directory (no server needed): autopilot traces,
//! coverage, screenshots and process logs written by run_moba_split_local.lua /
//! run_moba_role_ue.lua. Reads files only; never starts or stops anything.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// Newest run directory under `<repo>/target/split-runs` and `<repo>/target/mvp-a3`.
pub fn latest_run(repo: &Path) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for parent in ["target/split-runs", "target/mvp-a3"] {
        let Ok(entries) = fs::read_dir(repo.join(parent)) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || !path.join("children.json").is_file() {
                continue;
            }
            let Ok(modified) = entry.metadata().and_then(|m| m.modified()) else { continue };
            if best.as_ref().is_none_or(|(time, _)| modified > *time) {
                best = Some((modified, path));
            }
        }
    }
    best.map(|(_, path)| path)
}

fn summarize_trace(path: &Path) -> Value {
    let Ok(text) = fs::read_to_string(path) else {
        return json!({ "trace": path.display().to_string(), "readable": false });
    };
    let mut events: BTreeMap<String, u64> = BTreeMap::new();
    let mut intents: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    let mut last_tick = 0u64;
    let mut finished = Value::Null;
    let mut last_reasons: Vec<Value> = Vec::new();
    for line in text.lines() {
        let Ok(row) = serde_json::from_str::<Value>(line) else { continue };
        let event = row["event"].as_str().unwrap_or("?").to_string();
        *events.entry(event.clone()).or_default() += 1;
        if let Some(tick) = row["tick"].as_u64() {
            last_tick = last_tick.max(tick);
        }
        let intent = row["autopilot_intent"].as_str().map(str::to_string);
        let outcome = match event.as_str() {
            "authority" => row["ack_result"].as_str().or(row["authority_result"].as_str()).map(|r| format!("ack_{r}")),
            "not_submitted" => Some("not_submitted".to_string()),
            "local_action" => Some("local".to_string()),
            _ => None,
        };
        if let (Some(intent), Some(outcome)) = (intent, outcome) {
            *intents.entry(intent).or_default().entry(outcome).or_default() += 1;
        }
        if event == "not_submitted" || event == "no_effect" {
            last_reasons.push(row.clone());
            if last_reasons.len() > 5 {
                last_reasons.remove(0);
            }
        }
        if event == "match_finished" {
            finished = row.clone();
        }
    }
    json!({
        "trace": path.display().to_string(),
        "lines": text.lines().count(),
        "last_tick": last_tick,
        "minutes_at_60hz": (last_tick as f64 / 3600.0 * 10.0).round() / 10.0,
        "events": events,
        "intents": intents,
        "match_finished": finished,
        "recent_problems": last_reasons,
    })
}

/// Status of every client in a run directory plus the newest screenshots.
pub fn run_status(dir: &Path) -> Value {
    let mut clients = Vec::new();
    let mut screenshots: Vec<(std::time::SystemTime, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        let mut subdirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        subdirs.sort();
        for sub in subdirs {
            let Ok(files) = fs::read_dir(&sub) else { continue };
            for file in files.flatten() {
                let path = file.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if name.starts_with("autopilot-") && name.ends_with(".jsonl") {
                    clients.push(summarize_trace(&path));
                } else if name.ends_with(".png") && name.contains("-shot-") {
                    if let Ok(modified) = file.metadata().and_then(|m| m.modified()) {
                        screenshots.push((modified, path.display().to_string()));
                    }
                }
            }
        }
    }
    screenshots.sort();
    let recent: Vec<String> = screenshots.iter().rev().take(4).map(|(_, p)| p.clone()).collect();
    let lifecycle = fs::read_to_string(dir.join("lifecycle.json")).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
    json!({
        "run": dir.display().to_string(),
        "ended": lifecycle.is_some(),
        "lifecycle": lifecycle,
        "clients": clients,
        "recent_screenshots": recent,
    })
}
