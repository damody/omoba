@echo off
REM ======================================================================
REM  run_stress.bat -- TD_STRESS 效能測試啟動器（RELEASE build）
REM
REM  步驟：
REM    1. 結束殘留的 omobab.exe / executor.exe
REM    2. 重新產生 scripts\lua_data\TD_STRESS\map.lua
REM    3. 備份 omb\game.toml，並暫時替換為 omb\game_stress.toml
REM    4. 只有過期時才 build base_content DLL (release) + omb backend (release)。
REM    5. 只有過期時才 build omfx executor (release)，然後執行。
REM    6. 啟動 release backend，再啟動 release frontend。
REM    7. 結束後一律清理 backend 並還原 omb\game.toml
REM ======================================================================

setlocal
pushd "%~dp0"

set FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1
set EXECUTOR=omfx\target\release\executor.exe
set BACKEND=omb\target\release\omobab.exe
set OMB_DLL_PATH=omb\scripts\base_content.dll
set OMB_GAME_TOML=omb\game.toml
set OMB_LUA_CONTENT=
set OMB_LUA_CONTENT_ROOT=
set OMB_LUA_HOT_RELOAD=
set OMB_STORY_DATA_DIR=scripts\lua_data

set TOML=omb\game.toml
set TOML_BAK=omb\game.toml.bak
set TOML_STRESS=omb\game_stress.toml

echo [0/7] Killing stale processes (if any)...
REM 不用 taskkill — 此機器上 taskkill/tasklist 會卡住數十秒不返回（疑似某個
REM Windows process enumeration API 路徑被 hook 卡住）。改走 PowerShell 的
REM Stop-Process，走不同 API 路徑、秒回。
powershell -NoProfile -Command "Stop-Process -Name 'omobab','executor' -Force -ErrorAction SilentlyContinue"

echo [1/7] Regenerating stress map...
REM 使用 Windows 官方 py launcher 而非 `python`，避免 PATH 上的 Microsoft Store
REM Microsoft Store python.exe stub 攔截
REM 並彈出 Store 對話框讓 cmd 卡死。
py -3 scripts\gen_stress_map.py
if %errorlevel% neq 0 (
    echo   Stress map generation failed!
    popd
    pause
    exit /b 1
)

echo [2/7] Switching game.toml to stress variant (backup at %TOML_BAK%)...
if not exist "%TOML_STRESS%" (
    echo   %TOML_STRESS% missing!
    popd
    pause
    exit /b 1
)
copy /y "%TOML%" "%TOML_BAK%" >nul
copy /y "%TOML_STRESS%" "%TOML%" >nul

call :main
set MAIN_ERR=%errorlevel%
goto :restore

:main
echo [3/7] Checking script DLL (scripts\base_content, release)...
call :ensure_fresh script-dll release "release script DLL" "cargo build --release --manifest-path scripts\Cargo.toml -p base_content" "Script DLL build failed!"
if errorlevel 1 exit /b 1

%FRESHNESS% -Action stage-dll -Profile release
if errorlevel 1 (
    echo   Script DLL staging failed!
    exit /b 1
)

echo [4/7] Checking backend (omb, release)...
call :ensure_fresh backend release "release backend" "cargo build --release --manifest-path omb\Cargo.toml -p omobab" "Backend build failed!"
if errorlevel 1 exit /b 1

echo [5/7] Checking frontend (omfx executor, release)...
call :ensure_fresh frontend release "release frontend" "cargo build --release --manifest-path omfx\Cargo.toml -p executor" "Frontend build failed!"
if errorlevel 1 exit /b 1

if not exist "%BACKEND%" (
    echo   Backend executable missing: %BACKEND%
    exit /b 1
)

if not exist "%EXECUTOR%" (
    echo   Frontend executable missing: %EXECUTOR%
    exit /b 1
)

echo [6/7] Starting backend (omobab, release)...
call :start_backend
if errorlevel 1 exit /b 1

echo [7/7] Running frontend (omfx executor, release)...
"%EXECUTOR%"
set RUN_ERR=%errorlevel%
call :stop_backend
exit /b %RUN_ERR%

:start_backend
set BACKEND_PID=
set BACKEND_PID_FILE=omb\log\launcher_backend.pid
if exist "%BACKEND_PID_FILE%" del "%BACKEND_PID_FILE%" >nul 2>&1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\start_backend.ps1 -Exe "%BACKEND%" -WorkingDirectory "omb" -PidFile "%BACKEND_PID_FILE%"
if errorlevel 1 (
    echo   Backend start failed!
    exit /b 1
)
if exist "%BACKEND_PID_FILE%" set /p BACKEND_PID=<"%BACKEND_PID_FILE%"
if not defined BACKEND_PID (
    echo   Backend start failed!
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
    echo   %FAIL_MSG%
    exit /b 1
)
exit /b 0

:restore
echo.
call :stop_backend
echo Restoring %TOML% from backup...
if exist "%TOML_BAK%" (
    copy /y "%TOML_BAK%" "%TOML%" >nul
    del "%TOML_BAK%" >nul 2>&1
)

popd
exit /b %MAIN_ERR%
