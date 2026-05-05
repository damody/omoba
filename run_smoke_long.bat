@echo off
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

echo [4/5] Set auto-smoke envs (start at 2s, exit at 60s)...
set OMFX_AUTO_START_AFTER_SEC=2
set OMFX_AUTO_EXIT_AFTER_SEC=60

echo [5/5] Run executor (auto-pressed + auto-exit)...
"%EXECUTOR%" 2>&1
set RUN_ERR=%errorlevel%
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
