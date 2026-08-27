@echo off
setlocal EnableDelayedExpansion
pushd "%~dp0"

set "FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1"
set "EXECUTOR=omfx\target\debug\executor.exe"
set "BACKEND=omb\target\debug\omobab.exe"
set "OMB_GAME_TOML=%CD%\omb\game.toml"
set "OMFX_GAME_TOML=%CD%\omfx\game.toml"
set "OMB_STORY=FOG_2TEAM_DEMO"
set "OMB_SCENE_PATH="

if not exist "scripts\lua_data\FOG_2TEAM_DEMO\map.lua" (
    echo Demo package missing: scripts\lua_data\FOG_2TEAM_DEMO
    goto :fail_pause
)

if not defined PROTOC (
    if exist "D:\MProfiler\profiler-core\tools\protoc\bin\protoc.exe" (
        set "PROTOC=D:\MProfiler\profiler-core\tools\protoc\bin\protoc.exe"
    )
)

echo [0/6] Preparing isolated fog demo launcher...

echo [1/6] Checking script DLL (scripts\base_content)...
call :ensure_fresh script-dll "script DLL" "cargo build --manifest-path scripts\Cargo.toml -p base_content --features runtime-lua-content" "Script DLL build failed!"
if errorlevel 1 goto :fail

%FRESHNESS% -Action stage-dll
if errorlevel 1 (
    echo Script DLL staging failed!
    goto :fail_pause
)

echo [2/6] Checking backend (omb)...
call :ensure_fresh backend "backend" "cargo build --manifest-path omb\Cargo.toml --features runtime-lua-content" "Backend build failed!"
if errorlevel 1 goto :fail

echo [3/6] Checking frontend (omfx executor)...
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

echo [4/6] Starting backend...
call :start_backend
if errorlevel 1 goto :fail

echo [5/6] Starting two frontends...
call :start_frontend 1 1 player1 20
if errorlevel 1 goto :fail
call :start_frontend 2 2 player2 980
if errorlevel 1 goto :fail

echo [6/6] Waiting for frontends...
call :wait_frontends
set "RUN_ERR=%errorlevel%"
call :stop_backend
popd
exit /b %RUN_ERR%

:start_backend
set "BACKEND_PID="
set "BACKEND_PID_FILE=omb\log\launcher_backend.pid"
if exist "%BACKEND_PID_FILE%" del "%BACKEND_PID_FILE%" >nul 2>&1
call powershell -NoProfile -ExecutionPolicy Bypass -File scripts\start_backend.ps1 -Exe "%BACKEND%" -WorkingDirectory "omb" -PidFile "%BACKEND_PID_FILE%"
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

:start_frontend
set "PLAYER_ID=%~1"
set "TEAM_ID=%~2"
set "PLAYER_NAME=%~3"
set "WINDOW_X=%~4"
set "FRONTEND_PID_FILE=omfx\target\frontend_%PLAYER_ID%.pid"
if exist "%FRONTEND_PID_FILE%" del "%FRONTEND_PID_FILE%" >nul 2>&1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\start_fog_demo_frontend.ps1 -Exe "%EXECUTOR%" -WorkingDirectory "omfx" -PidFile "%FRONTEND_PID_FILE%" -PlayerId %PLAYER_ID% -TeamId %TEAM_ID% -PlayerName "%PLAYER_NAME%" -WindowX %WINDOW_X%
if errorlevel 1 (
    echo Frontend %PLAYER_ID% start failed!
    exit /b 1
)
set /p FRONTEND_PID_%PLAYER_ID%=<"%FRONTEND_PID_FILE%"
echo   -^> frontend %PLAYER_ID% team %TEAM_ID% PID !FRONTEND_PID_%PLAYER_ID%!
exit /b 0

:wait_frontends
set "RUN_ERR=0"
:wait_loop
set "FRONTEND_ALIVE=0"
for %%P in (!FRONTEND_PID_1! !FRONTEND_PID_2!) do powershell -NoProfile -Command "if (Get-Process -Id %%P -ErrorAction SilentlyContinue) { exit 0 } exit 1" && set "FRONTEND_ALIVE=1"
if "%FRONTEND_ALIVE%"=="0" goto :frontends_done
if defined BACKEND_PID (
    powershell -NoProfile -ExecutionPolicy Bypass -Command "if (Get-Process -Id %BACKEND_PID% -ErrorAction SilentlyContinue) { exit 0 } exit 1"
    if errorlevel 1 (
        echo Backend exited; stopping frontend clients...
        call :stop_frontends
        exit /b 1
    )
)
powershell -NoProfile -Command "Start-Sleep -Milliseconds 500"
goto :wait_loop

:frontends_done
exit /b %RUN_ERR%

:stop_frontends
for %%P in (!FRONTEND_PID_1! !FRONTEND_PID_2!) do powershell -NoProfile -Command "Stop-Process -Id %%P -Force -ErrorAction SilentlyContinue"
exit /b 0

:stop_backend
if defined BACKEND_PID (
    echo Stopping backend PID %BACKEND_PID%...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Stop-Process -Id %BACKEND_PID% -Force -ErrorAction SilentlyContinue"
    if defined BACKEND_PID_FILE if exist "%BACKEND_PID_FILE%" del "%BACKEND_PID_FILE%" >nul 2>&1
    set "BACKEND_PID="
)
exit /b 0

:ensure_fresh
set "ARTIFACT=%~1"
set "LABEL=%~2"
set "BUILD_CMD=%~3"
set "FAIL_MSG=%~4"

%FRESHNESS% -Action check -Artifact %ARTIFACT%
set "FRESH_ERR=%errorlevel%"
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
