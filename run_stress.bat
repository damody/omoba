@echo off
REM ======================================================================
REM  run_stress.bat -- TD_STRESS 效能測試啟動器（RELEASE build）
REM
REM  步驟：
REM    1. 結束殘留的 omobab.exe / executor.exe
REM    2. 重新產生 scripts\lua_data\TD_STRESS\map.lua
REM    3. 備份 omb\game.toml，並暫時替換為 omb\game_stress.toml
REM    4. 只有過期時才 build base_content DLL (release) + omb backend (release)。
REM       omfx hard-code target\debug\omobab.exe，所以把 release exe 複製覆蓋
REM       target\debug\omobab.exe，讓 omfx 使用快速 build。
REM    5. 只有過期時才 build omfx executor (release)，然後執行。
REM    6. 結束後一律還原 omb\game.toml
REM ======================================================================

setlocal
pushd %~dp0

set FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1
set EXECUTOR=omfx\target\release\executor.exe
set OMB_LUA_CONTENT=
set OMB_LUA_CONTENT_ROOT=
set OMB_STORY_DATA_DIR=

set TOML=omb\game.toml
set TOML_BAK=omb\game.toml.bak
set TOML_STRESS=omb\game_stress.toml

echo [0/6] Killing stale processes (if any)...
REM 不用 taskkill — 此機器上 taskkill/tasklist 會卡住數十秒不返回（疑似某個
REM Windows process enumeration API 路徑被 hook 卡住）。改走 PowerShell 的
REM Stop-Process，走不同 API 路徑、秒回。
powershell -NoProfile -Command "Stop-Process -Name 'omobab','executor' -Force -ErrorAction SilentlyContinue"

echo [1/6] Regenerating stress map...
REM 使用 Windows 官方 py launcher 而非 `python`，避免 PATH 上的 Microsoft Store
REM stub (C:\Users\<user>\AppData\Local\Microsoft\WindowsApps\python.exe) 攔截
REM 並彈出 Store 對話框讓 cmd 卡死。
py -3 scripts\gen_stress_map.py
if %errorlevel% neq 0 (
    echo   Stress map generation failed!
    popd
    pause
    exit /b 1
)

echo [2/6] Switching game.toml to stress variant (backup at %TOML_BAK%)...
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
echo [3/6] Checking script DLL (scripts\base_content, release)...
call :ensure_fresh script-dll release "release script DLL" "cargo build --release --manifest-path scripts\Cargo.toml -p base_content" "Script DLL build failed!"
if errorlevel 1 exit /b 1

%FRESHNESS% -Action stage-dll -Profile release
if errorlevel 1 (
    echo   Script DLL staging failed!
    exit /b 1
)

echo [4/6] Checking backend (omb, release)...
call :ensure_fresh backend release "release backend" "cargo build --release --manifest-path omb\Cargo.toml -p omobab" "Backend build failed!"
if errorlevel 1 exit /b 1

REM omfx 會 spawn target\debug\omobab.exe；用 release exe 覆蓋它，
REM 讓 perf test 實際跑 optimized build。
%FRESHNESS% -Action stage-backend-spawn
if errorlevel 1 (
    echo   Backend spawn staging failed!
    exit /b 1
)

echo [5/6] Checking frontend (omfx executor, release)...
call :ensure_fresh frontend release "release frontend" "cargo build --release --manifest-path omfx\Cargo.toml -p executor" "Frontend build failed!"
if errorlevel 1 exit /b 1

if not exist "%EXECUTOR%" (
    echo   Frontend executable missing: %EXECUTOR%
    exit /b 1
)

echo [6/6] Running frontend (omfx executor, release; spawns omb child)...
"%EXECUTOR%"
exit /b %errorlevel%

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
echo Restoring %TOML% from backup...
if exist "%TOML_BAK%" (
    copy /y "%TOML_BAK%" "%TOML%" >nul
    del "%TOML_BAK%" >nul 2>&1
)

popd
exit /b %MAIN_ERR%
