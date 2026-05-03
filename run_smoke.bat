@echo off
REM ======================================================================
REM run_smoke.bat — automated smoke run.
REM Auto-presses Start Round at t=2s, exits at t=10s. Reads game.toml
REM STORY as-is; assume TD_1 unless caller already swapped it.
REM
REM `setlocal` keeps the OMFX_AUTO_* env vars scoped to this script —
REM otherwise they leak into the parent cmd and a subsequent `run.bat`
REM in the same window would also auto-exit at 10s, which would look
REM exactly like the game freezing.
REM
REM Output:
REM   - omfx_app.log      (omfx + sim_runner side)
REM   - omb/log/requests.log  (omb host side; appends, very large)
REM ======================================================================

setlocal

pushd %~dp0

echo [0/3] Killing stale processes...
powershell -NoProfile -Command "Stop-Process -Name 'omobab','executor' -Force -ErrorAction SilentlyContinue"

echo [1/3] Build script DLL + omb backend...
cargo build --manifest-path scripts\Cargo.toml -p base_content
if %errorlevel% neq 0 ( echo Script DLL build failed!& popd & exit /b 1 )
if not exist omb\scripts mkdir omb\scripts
copy /y scripts\target\debug\base_content.dll omb\scripts\base_content.dll >nul
cargo build --manifest-path omb\Cargo.toml
if %errorlevel% neq 0 ( echo Backend build failed!& popd & exit /b 1 )

echo [2/3] Set auto-smoke envs (start at 2s, exit at 10s)...
set OMFX_AUTO_START_AFTER_SEC=2
set OMFX_AUTO_EXIT_AFTER_SEC=10

echo [3/3] Run executor (auto-pressed + auto-exit)...
cargo run --manifest-path omfx\Cargo.toml -p executor

echo.
echo ===== smoke run complete =====
popd
