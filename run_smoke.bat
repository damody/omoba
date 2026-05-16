@echo off
REM ======================================================================
REM run_smoke.bat — 自動化 smoke run。
REM t=2s 自動按 Start Round，t=10s 結束。直接讀取 game.toml 的
REM STORY；除非 caller 已替換，否則假設為 TD_1。
REM
REM `setlocal` 讓 OMFX_AUTO_* env vars 只作用於此 script；
REM 否則它們會 leak 到 parent cmd，導致同一視窗後續執行的 `run.bat`
REM 也在 10s 自動結束，看起來就像遊戲 freeze。
REM
REM 輸出：
REM   - omfx_app.log      (omfx + sim_runner side)
REM   - omb/log/requests.log  (omb host side；append，可能很大)
REM ======================================================================

setlocal

pushd "%~dp0"

set FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1
set EXECUTOR=omfx\target\debug\executor.exe
set BACKEND=omb\target\debug\omobab.exe
set OMB_DLL_PATH=scripts\base_content.dll
set OMB_SCRIPTS_DIR=scripts
set OMB_GAME_TOML=scripts\game.toml
set OMB_LUA_CONTENT=1
set OMB_LUA_CONTENT_ROOT=scripts\lua_data
set OMB_STORY_DATA_DIR=%OMB_LUA_CONTENT_ROOT%

echo [0/6] Killing stale processes...
powershell -NoProfile -Command "Stop-Process -Name 'omobab','executor' -Force -ErrorAction SilentlyContinue"

echo [1/6] Checking script DLL (debug)...
call :ensure_fresh script-dll debug "script DLL" "cargo build --manifest-path scripts\Cargo.toml -p base_content --features runtime-lua-content" "Script DLL build failed!"
if errorlevel 1 goto :fail

%FRESHNESS% -Action stage-dll -Profile debug
if errorlevel 1 (
    echo Script DLL staging failed!
    goto :fail
)

echo [2/6] Checking backend (debug)...
call :ensure_fresh backend debug "backend" "cargo build --manifest-path omb\Cargo.toml --features runtime-lua-content" "Backend build failed!"
if errorlevel 1 goto :fail

echo [3/6] Checking frontend (debug)...
call :ensure_fresh frontend debug "frontend" "cargo build --manifest-path omfx\Cargo.toml -p executor --features runtime-lua-content" "Frontend build failed!"
if errorlevel 1 goto :fail

if not exist "%BACKEND%" (
    echo Backend executable missing: %BACKEND%
    goto :fail
)

if not exist "%EXECUTOR%" (
    echo Frontend executable missing: %EXECUTOR%
    goto :fail
)

echo [4/6] Set auto-smoke envs (start at 2s, exit at 10s)...
set OMFX_AUTO_START_AFTER_SEC=2
set OMFX_AUTO_EXIT_AFTER_SEC=10

echo [5/6] Starting backend...
call :start_backend
if errorlevel 1 goto :fail

echo [6/6] Run executor (auto-pressed + auto-exit)...
"%EXECUTOR%"
set RUN_ERR=%errorlevel%
call :stop_backend

echo.
echo ===== smoke run complete =====
popd
exit /b %RUN_ERR%

:start_backend
set BACKEND_PID=
set BACKEND_PID_FILE=omb\log\launcher_backend.pid
if exist "%BACKEND_PID_FILE%" del "%BACKEND_PID_FILE%" >nul 2>&1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\start_backend.ps1 -Exe "%BACKEND%" -WorkingDirectory "omb" -PidFile "%BACKEND_PID_FILE%"
if errorlevel 1 (
    echo Backend start failed!
    exit /b 1
)
if exist "%BACKEND_PID_FILE%" set /p BACKEND_PID=<"%BACKEND_PID_FILE%"
if not defined BACKEND_PID (
    echo Backend start failed!
    exit /b 1
)
echo   -^> backend PID %BACKEND_PID%
powershell -NoProfile -Command "Start-Sleep -Milliseconds 500"
exit /b 0

:stop_backend
if defined BACKEND_PID (
    echo Stopping backend PID %BACKEND_PID%...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Stop-Process -Id %BACKEND_PID% -Force -ErrorAction SilentlyContinue"
    if defined BACKEND_PID_FILE if exist "%BACKEND_PID_FILE%" del "%BACKEND_PID_FILE%" >nul 2>&1
    set BACKEND_PID=
)
exit /b 0

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
    echo   -^> freshness check failed for %LABEL%; aborting.
    exit /b 1
)

%BUILD_CMD%
if errorlevel 1 (
    echo %FAIL_MSG%
    exit /b 1
)
exit /b 0

:fail
call :stop_backend
popd
exit /b 1
