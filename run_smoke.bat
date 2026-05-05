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

set FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1
set EXECUTOR=omfx\target\debug\executor.exe

echo [0/5] Killing stale processes...
powershell -NoProfile -Command "Stop-Process -Name 'omobab','executor' -Force -ErrorAction SilentlyContinue"

echo [1/5] Checking script DLL (debug)...
call :ensure_fresh script-dll debug "script DLL" "cargo build --manifest-path scripts\Cargo.toml -p base_content" "Script DLL build failed!"
if errorlevel 1 goto :fail

%FRESHNESS% -Action stage-dll -Profile debug
if errorlevel 1 (
    echo Script DLL staging failed!
    goto :fail
)

echo [2/5] Checking backend (debug)...
call :ensure_fresh backend debug "backend" "cargo build --manifest-path omb\Cargo.toml" "Backend build failed!"
if errorlevel 1 goto :fail

echo [3/5] Checking frontend (debug)...
call :ensure_fresh frontend debug "frontend" "cargo build --manifest-path omfx\Cargo.toml -p executor" "Frontend build failed!"
if errorlevel 1 goto :fail

if not exist "%EXECUTOR%" (
    echo Frontend executable missing: %EXECUTOR%
    goto :fail
)

echo [4/5] Set auto-smoke envs (start at 2s, exit at 10s)...
set OMFX_AUTO_START_AFTER_SEC=2
set OMFX_AUTO_EXIT_AFTER_SEC=10

echo [5/5] Run executor (auto-pressed + auto-exit)...
"%EXECUTOR%"
set RUN_ERR=%errorlevel%

echo.
echo ===== smoke run complete =====
popd
exit /b %RUN_ERR%

:ensure_fresh
set ARTIFACT=%~1
set PROFILE=%~2
set LABEL=%~3
set BUILD_CMD=%~4
set FAIL_MSG=%~5

%FRESHNESS% -Action check -Artifact %ARTIFACT% -Profile %PROFILE%
set FRESH_ERR=%errorlevel%
if "%FRESH_ERR%"=="0" (
    echo   -^> %LABEL% up-to-date; skipping build.
    exit /b 0
)
if "%FRESH_ERR%"=="1" (
    echo   -^> %LABEL% stale; building...
) else (
    echo   -^> freshness check failed for %LABEL%; building...
)

%BUILD_CMD%
if errorlevel 1 (
    echo %FAIL_MSG%
    exit /b 1
)
exit /b 0

:fail
popd
exit /b 1
