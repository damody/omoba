@echo off
setlocal
pushd "%~dp0"

set FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1
set EXECUTOR=omfx\target\debug\executor.exe
set BACKEND=omb\target\debug\omobab.exe
set OMB_DLL_PATH=scripts\base_content.dll
set OMB_GAME_TOML=scripts\game.toml
set OMB_LUA_CONTENT=1
set OMB_LUA_CONTENT_ROOT=scripts\lua_data
set OMB_LUA_HOT_RELOAD=1
set OMB_STORY_DATA_DIR=%OMB_LUA_CONTENT_ROOT%
set OMB_STORY=TD_1
set OMB_SCENE_PATH=%OMB_STORY_DATA_DIR%\%OMB_STORY%

echo [0/5] Killing stale processes (if any)...
taskkill /f /im omobab.exe >nul 2>&1
taskkill /f /im executor.exe >nul 2>&1

echo [1/5] Checking script DLL (scripts\base_content)...
call :ensure_fresh script-dll "script DLL" "cargo build --manifest-path scripts\Cargo.toml -p base_content --features runtime-lua-content" "Script DLL build failed!"
if errorlevel 1 goto :fail

%FRESHNESS% -Action stage-dll
if errorlevel 1 (
    echo Script DLL staging failed!
    goto :fail_pause
)

echo [2/5] Checking backend (omb)...
call :ensure_fresh backend "backend" "cargo build --manifest-path omb\Cargo.toml --features runtime-lua-content" "Backend build failed!"
if errorlevel 1 goto :fail

echo [3/5] Checking frontend (omfx executor)...
call :ensure_fresh frontend "frontend" "cargo build --manifest-path omfx\Cargo.toml -p executor --features runtime-lua-content" "Frontend build failed!"
if errorlevel 1 goto :fail

if not exist "%BACKEND%" (
    echo Backend executable missing: %BACKEND%
    goto :fail_pause
)

if not exist "%EXECUTOR%" (
    echo Frontend executable missing: %EXECUTOR%
    goto :fail_pause
)

echo [4/5] Starting backend...
call :start_backend
if errorlevel 1 goto :fail

echo [5/5] Running frontend...
"%EXECUTOR%"
set RUN_ERR=%errorlevel%
call :stop_backend
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
set LABEL=%~2
set BUILD_CMD=%~3
set FAIL_MSG=%~4

%FRESHNESS% -Action check -Artifact %ARTIFACT%
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
    pause
    exit /b 1
)
exit /b 0

:fail_pause
pause

:fail
call :stop_backend
popd
exit /b 1
