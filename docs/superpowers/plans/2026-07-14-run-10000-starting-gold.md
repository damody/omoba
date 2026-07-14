# Run 10000 Starting Gold Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a test-only `run_10000.bat` launcher that gives every TD difficulty 10,000 starting gold without changing the normal 650 starting-gold profiles.

**Architecture:** Keep difficulty profiles authoritative in `TdDifficultyConfig`, then apply one optional `OMB_TD_STARTING_GOLD` environment override after difficulty selection. The new batch launcher sets that override and delegates all building, staging, argument forwarding, and process startup to the existing `run.bat`.

**Tech Stack:** Rust 1.95.0, Cargo tests, Windows batch, PowerShell verification

---

### Task 1: Add a tested starting-gold override to TD configuration

**Files:**
- Modify: `omoba-core/src/runtime/native/initialization.rs`
- Test: `omoba-core/src/runtime/native/initialization.rs`

**Step 1: Write the failing tests**

Add these tests beside the existing `TdDifficultyConfig` profile tests:

```rust
#[test]
fn td_starting_gold_override_applies_to_every_difficulty() {
    for difficulty in ["novice", "intermediate", "advanced", "expert"] {
        let config = apply_starting_gold_override(
            TdDifficultyConfig::from_config_value(difficulty),
            Some("10000"),
        );

        assert_eq!(config.starting_gold, 10_000, "{difficulty}");
    }
}

#[test]
fn invalid_td_starting_gold_override_preserves_profile_default() {
    for value in [None, Some(""), Some("not-a-number"), Some("-1")] {
        let config = apply_starting_gold_override(
            TdDifficultyConfig::from_config_value("novice"),
            value,
        );

        assert_eq!(config.starting_gold, 650, "{value:?}");
    }
}
```

These tests call a pure helper instead of modifying the process environment, so they remain safe when Rust runs tests concurrently.

**Step 2: Run the focused test and verify it fails**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml td_starting_gold_override
```

Expected: compilation fails because `apply_starting_gold_override` does not exist yet.

**Step 3: Implement the minimal override logic**

Add the environment-variable name near `TD_DIFFICULTY_ENV`:

```rust
const TD_STARTING_GOLD_ENV: &str = "OMB_TD_STARTING_GOLD";
```

Add this pure helper near `TdDifficultyConfig`:

```rust
fn apply_starting_gold_override(
    mut config: TdDifficultyConfig,
    override_value: Option<&str>,
) -> TdDifficultyConfig {
    if let Some(starting_gold) = override_value
        .and_then(|value| value.trim().parse::<i32>().ok())
        .filter(|value| *value >= 0)
    {
        config.starting_gold = starting_gold;
    }

    config
}
```

Change `TdDifficultyConfig::from_env()` so difficulty selection happens first and the optional override is applied afterward:

```rust
fn from_env() -> Self {
    let config = std::env::var(TD_DIFFICULTY_ENV)
        .ok()
        .map(|value| Self::from_config_value(&value))
        .unwrap_or(Self::EXPERT);
    let starting_gold_override = std::env::var(TD_STARTING_GOLD_ENV).ok();

    apply_starting_gold_override(config, starting_gold_override.as_deref())
}
```

Do not change the four profile constants: their normal starting gold must remain `650`.

**Step 4: Run the focused tests and verify they pass**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml td_starting_gold_override
```

Expected: both new tests pass.

**Step 5: Run all initialization tests**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml runtime::native::initialization::tests
```

Expected: existing difficulty-profile tests still pass with `650`, and the new override tests pass with `10_000`.

**Step 6: Commit the Rust change**

```powershell
git add omoba-core/src/runtime/native/initialization.rs
git commit -m "feat: allow test starting gold override"
```

### Task 2: Add the Windows test launcher

**Files:**
- Create: `run_10000.bat`

**Step 1: Run a precondition check and verify it fails**

Run:

```powershell
if (Test-Path 'run_10000.bat') { throw 'run_10000.bat already exists; inspect it before continuing' } else { throw 'Expected failure: run_10000.bat has not been created yet' }
```

Expected: the command fails with `Expected failure: run_10000.bat has not been created yet`.

**Step 2: Create the minimal delegating launcher**

Create `run_10000.bat` with this exact content:

```bat
@echo off
setlocal
set "OMB_TD_STARTING_GOLD=10000"
call "%~dp0run.bat" %*
set "RUN_ERR=%errorlevel%"
exit /b %RUN_ERR%
```

This preserves all existing `run.bat` behavior, forwards arguments such as `--trace`, and returns the delegated launcher's exit code.

**Step 3: Convert the launcher to CRLF without a UTF-8 BOM**

Run:

```powershell
$p = 'D:\code\omoba\run_10000.bat'; $c = (Get-Content -Raw $p) -replace "(?<!`r)`n","`r`n"; [System.IO.File]::WriteAllText($p, $c, (New-Object System.Text.UTF8Encoding $false))
```

Expected: the file remains textually identical, with CRLF on every line and no BOM.

**Step 4: Verify launcher contents, delegation, and line endings**

Run:

```powershell
$p = 'run_10000.bat'; $bytes = [IO.File]::ReadAllBytes($p); $text = [Text.Encoding]::UTF8.GetString($bytes); if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) { throw 'UTF-8 BOM found' }; if ($text -match "(?<!`r)`n") { throw 'LF-only line ending found' }; foreach ($required in @('set "OMB_TD_STARTING_GOLD=10000"', 'call "%~dp0run.bat" %*', 'exit /b %RUN_ERR%')) { if (-not $text.Contains($required)) { throw "Missing: $required" } }; 'run_10000.bat verification passed'
```

Expected: prints `run_10000.bat verification passed`.

**Step 5: Commit the launcher**

```powershell
git add run_10000.bat
git commit -m "feat: add 10000 gold test launcher"
```

### Task 3: Run final regression verification

**Files:**
- Verify: `omoba-core/src/runtime/native/initialization.rs`
- Verify: `run_10000.bat`

**Step 1: Verify Rust formatting**

Run:

```powershell
cargo fmt --manifest-path omoba-core/Cargo.toml -- --check
```

Expected: exits successfully without formatting differences.

**Step 2: Run the complete `omoba-core` test suite**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml
```

Expected: all tests pass.

**Step 3: Re-run the batch-file verification**

Run:

```powershell
$p = 'run_10000.bat'; $bytes = [IO.File]::ReadAllBytes($p); $text = [Text.Encoding]::UTF8.GetString($bytes); if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) { throw 'UTF-8 BOM found' }; if ($text -match "(?<!`r)`n") { throw 'LF-only line ending found' }; foreach ($required in @('set "OMB_TD_STARTING_GOLD=10000"', 'call "%~dp0run.bat" %*', 'exit /b %RUN_ERR%')) { if (-not $text.Contains($required)) { throw "Missing: $required" } }; 'run_10000.bat verification passed'
```

Expected: prints `run_10000.bat verification passed`.

**Step 4: Inspect the final repository state**

Run:

```powershell
git status --short
git log --oneline -5
```

Expected: only the pre-existing untracked `omfue` entry may remain; the implementation files are committed. Do not add, delete, or modify `omfue`.
