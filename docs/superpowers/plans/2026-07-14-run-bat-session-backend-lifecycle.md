# `run.bat` Session Backend Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `run.bat` leave backend ownership to the omfx session launcher so every game after returning to the title starts with a fresh backend world.

**Architecture:** Add a standalone PowerShell regression check for the Windows launcher contract, then remove the prestarted external-backend path from `run.bat`. omfx will keep receiving `OMFX_BACKEND_EXE` and will own the backend child for each selected session, while `run_10000.bat` continues to delegate and pass its environment override through unchanged.

**Tech Stack:** Windows batch, PowerShell, Rust 1.95.0, Cargo tests, Fyrox omfx frontend

---

## File Structure

- Create `scripts/test_run_session_launcher.ps1`: a dependency-free static regression check for backend ownership, required launcher wiring, CRLF, and UTF-8 BOM rules.
- Modify `run.bat`: build artifacts and launch only the frontend; remove the external backend start/stop helpers.
- Do not modify `run_10000.bat`, omfx Rust sources, or `omfue`.

### Task 1: Capture the Conflicting Backend Ownership

**Files:**
- Create: `scripts/test_run_session_launcher.ps1`
- Test: `scripts/test_run_session_launcher.ps1`

- [ ] **Step 1: Write the failing launcher regression test**

Create `scripts/test_run_session_launcher.ps1` with:

```powershell
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$launcherPath = Join-Path $repoRoot 'run.bat'
$bytes = [System.IO.File]::ReadAllBytes($launcherPath)
$text = [System.Text.Encoding]::UTF8.GetString($bytes)

if ($bytes.Length -ge 3 -and
    $bytes[0] -eq 0xEF -and
    $bytes[1] -eq 0xBB -and
    $bytes[2] -eq 0xBF) {
    throw 'run.bat must not contain a UTF-8 BOM'
}

if ($text -match "(?<!`r)`n") {
    throw 'run.bat must use CRLF line endings'
}

foreach ($required in @(
    'set "OMFX_BACKEND_EXE=%CD%\%BACKEND%"',
    'echo   -^> frontend session launcher will start backend: %OMFX_BACKEND_EXE%',
    '"%EXECUTOR%"'
)) {
    if (-not $text.Contains($required)) {
        throw "run.bat is missing required session-launcher wiring: $required"
    }
}

foreach ($forbidden in @(
    'call :start_backend',
    'call :stop_backend',
    ':start_backend',
    ':stop_backend'
)) {
    if ($text.Contains($forbidden)) {
        throw "run.bat must not manage an external backend: $forbidden"
    }
}

Write-Output 'run.bat session launcher verification passed'
```

- [ ] **Step 2: Run the regression test and verify RED**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test_run_session_launcher.ps1
```

Expected: command exits non-zero with `run.bat must not manage an external backend: call :start_backend`.

- [ ] **Step 3: Commit the failing regression test**

```powershell
git add scripts/test_run_session_launcher.ps1
git commit -m "test: guard run session backend ownership"
```

### Task 2: Restore Session-Owned Backend Lifecycle

**Files:**
- Modify: `run.bat:69-111`
- Test: `scripts/test_run_session_launcher.ps1`

- [ ] **Step 1: Remove external backend startup and shutdown**

Replace the launch section after executable validation with:

```bat
echo [4/4] Running frontend...
echo   -^> frontend session launcher will start backend: %OMFX_BACKEND_EXE%
"%EXECUTOR%"
set "RUN_ERR=%errorlevel%"
popd
exit /b %RUN_ERR%
```

Delete the complete `:start_backend` and `:stop_backend` label blocks. Replace the final failure block with:

```bat
:fail_pause
pause

:fail
popd
exit /b 1
```

- [ ] **Step 2: Restore mandatory CRLF encoding**

Run:

```powershell
$p = 'D:\code\omoba\run.bat'
$c = (Get-Content -Raw $p) -replace "(?<!`r)`n", "`r`n"
[System.IO.File]::WriteAllText($p, $c, (New-Object System.Text.UTF8Encoding $false))
```

Expected: every line ends in CRLF and the file has no UTF-8 BOM.

- [ ] **Step 3: Run the regression test and verify GREEN**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test_run_session_launcher.ps1
```

Expected: command exits 0 and prints `run.bat session launcher verification passed`.

- [ ] **Step 4: Inspect the focused diff**

Run:

```powershell
git diff --check -- run.bat scripts/test_run_session_launcher.ps1
git diff -- run.bat scripts/test_run_session_launcher.ps1
```

Expected: only the external backend lifecycle is removed from `run.bat`; build, staging, stale-process cleanup, executable validation, frontend launch, and failure exit behavior remain.

- [ ] **Step 5: Commit the lifecycle fix**

```powershell
git add run.bat
git commit -m "fix: let game sessions own backend process"
```

### Task 3: Verify Frontend Session Shutdown and Integration

**Files:**
- Verify: `run.bat`
- Verify: `run_10000.bat`
- Verify: `omfx/game/src/backend_session.rs`
- Verify: `omfx/game/src/native.rs`

- [ ] **Step 1: Run focused backend ownership tests**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx backend_session::tests
```

Expected: all backend-session tests pass, including owned-child environment and idempotent external shutdown coverage.

- [ ] **Step 2: Run the return-to-title lifecycle regression**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx in_game_return_button_click_returns_to_main_menu
```

Expected: the return test passes and confirms lockstep, simulation, backend, and per-session frontend state are cleared.

- [ ] **Step 3: Re-run launcher and wrapper verification**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test_run_session_launcher.ps1
$wrapper = Get-Content -Raw run_10000.bat
if (-not $wrapper.Contains('call "%~dp0run.bat" %*')) { throw 'run_10000.bat no longer delegates to run.bat' }
if (-not $wrapper.Contains('set "OMB_TD_STARTING_GOLD=10000"')) { throw 'run_10000.bat lost its starting-gold override' }
Write-Output 'run_10000.bat delegation verification passed'
```

Expected: both verification messages are printed and the command exits 0.

- [ ] **Step 4: Inspect repository scope**

Run:

```powershell
git status --short
git log -5 --oneline
```

Expected: implementation files are committed; only the pre-existing `? omfue` submodule state remains. No file under `omfue` is staged or modified by this change.

- [ ] **Step 5: Manual smoke path**

Run `run.bat`, start any TD map, return to the title before or after `GameStart`, then immediately start another map. Expected: each Start creates one `omobab.exe`; Return terminates it; the second game begins at tick and round zero using the new selection.
