@echo off
setlocal EnableDelayedExpansion
pushd "%~dp0"

set "FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1"
set "EXECUTOR=omfx\target\debug\executor.exe"
set "BACKEND=omb\target\debug\omobab.exe"
set "CLIENT_RUNTIME=omoba-client-runtime\target\debug\omoba-client-runtime.exe"
set "RUN_MODE=%~1"
if not defined RUN_MODE set "RUN_MODE=visual"
set "FRONTEND_PID_1=0"
set "FRONTEND_PID_2=0"
set "SERVER_ADDR=127.0.0.1:50061"
set "PRESENTATION_1=127.0.0.1:62001"
set "PRESENTATION_2=127.0.0.1:62002"
if not defined OMOBA_RUN_ID set "OMOBA_RUN_ID=run-%RANDOM%-%RANDOM%"
set "OMOBA_RUN_ID=%OMOBA_RUN_ID: =0%"
set "EVIDENCE_DIR=%CD%\openspec\changes\extract-client-runtime-three-process-fog-validation\evidence\three-process-fog\runs\%OMOBA_RUN_ID%"
set "OMOBA_FOG_EVIDENCE_DIR=%EVIDENCE_DIR%"
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

echo [0/8] Preparing isolated fog demo launcher mode=%RUN_MODE%...
powershell -NoProfile -Command "$ports=50061,62001,62002; foreach($port in $ports){if(Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue){Write-Error ('port already in use: '+$port);exit 1}}"
if errorlevel 1 goto :fail

echo [1/8] Checking script DLL (scripts\base_content)...
call :ensure_fresh script-dll "script DLL" "cargo build --manifest-path scripts\Cargo.toml -p base_content --features runtime-lua-content" "Script DLL build failed!"
if errorlevel 1 goto :fail

%FRESHNESS% -Action stage-dll
if errorlevel 1 (
    echo Script DLL staging failed!
    goto :fail_pause
)

echo [2/8] Checking backend (omb)...
call :ensure_fresh backend "backend" "cargo build --manifest-path omb\Cargo.toml --features runtime-lua-content" "Backend build failed!"
if errorlevel 1 goto :fail

echo [3/8] Checking external client runtime...
cargo build --manifest-path omoba-client-runtime\Cargo.toml
if errorlevel 1 goto :fail

if /I "%RUN_MODE%"=="headless" goto :skip_frontend_build
echo [4/8] Checking frontend (omfx executor)...
call :ensure_fresh frontend "frontend" "cargo build --manifest-path omfx\Cargo.toml -p executor --features runtime-lua-content" "Frontend build failed!"
if errorlevel 1 goto :fail
:skip_frontend_build

if not exist "%BACKEND%" (
    echo Backend executable missing: %BACKEND%
    goto :fail_pause
)

if /I not "%RUN_MODE%"=="headless" if not exist "%EXECUTOR%" (
    echo Frontend executable missing: %EXECUTOR%
    goto :fail_pause
)

if not exist "%CLIENT_RUNTIME%" (
    echo Client runtime executable missing: %CLIENT_RUNTIME%
    goto :fail_pause
)

echo [5/8] Starting backend...
call :start_backend
if errorlevel 1 goto :fail

echo [6/8] Starting Team 1 and Team 2 runtime processes...
call :start_runtime 1 1 player1 "%PRESENTATION_1%"
if errorlevel 1 goto :fail
call :start_runtime 2 2 player2 "%PRESENTATION_2%"
if errorlevel 1 goto :fail

if /I "%RUN_MODE%"=="headless" goto :headless_wait

echo [7/8] Starting two renderer-only frontends...
call :start_frontend 1 1 player1 20 "%PRESENTATION_1%"
if errorlevel 1 goto :fail
call :start_frontend 2 2 player2 980 "%PRESENTATION_2%"
if errorlevel 1 goto :fail
call :write_manifest visual
if errorlevel 1 goto :fail
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\capture_fog_screenshots.ps1 -EvidenceDir "%EVIDENCE_DIR%" -Team1RendererPid !FRONTEND_PID_1! -Team2RendererPid !FRONTEND_PID_2!
if errorlevel 1 goto :fail
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dump_process_memory.ps1 -ProcessId !RUNTIME_PID_1! -ExpectedExe "%CLIENT_RUNTIME%" -OutputPath "%EVIDENCE_DIR%\team-1-runtime.dmp" -Role runtime
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dump_process_memory.ps1 -ProcessId !RUNTIME_PID_2! -ExpectedExe "%CLIENT_RUNTIME%" -OutputPath "%EVIDENCE_DIR%\team-2-runtime.dmp" -Role runtime
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dump_process_memory.ps1 -ProcessId !FRONTEND_PID_1! -ExpectedExe "%EXECUTOR%" -OutputPath "%EVIDENCE_DIR%\team-1-renderer.dmp" -Role renderer
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dump_process_memory.ps1 -ProcessId !FRONTEND_PID_2! -ExpectedExe "%EXECUTOR%" -OutputPath "%EVIDENCE_DIR%\team-2-renderer.dmp" -Role renderer
if "%OMOBA_RUN_LIFECYCLE%"=="1" powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run_fog_lifecycle.ps1 -ServerPid !BACKEND_PID! -Team1RuntimePid !RUNTIME_PID_1! -Team2RuntimePid !RUNTIME_PID_2! -Team1RendererPid !FRONTEND_PID_1! -RuntimeExe "%CLIENT_RUNTIME%" -RendererExe "%EXECUTOR%" -EvidenceDir "%EVIDENCE_DIR%"
if defined OMOBA_VISUAL_SECONDS powershell -NoProfile -Command "Start-Sleep -Seconds %OMOBA_VISUAL_SECONDS%"
if defined OMOBA_VISUAL_SECONDS call :stop_frontends

echo [8/8] Waiting for frontends...
call :wait_frontends
set "RUN_ERR=%errorlevel%"
call :stop_runtimes
call :stop_backend
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\compare_fog_evidence.ps1 -EvidenceDir "%EVIDENCE_DIR%"
if errorlevel 1 set "RUN_ERR=%errorlevel%"
popd
exit /b %RUN_ERR%

:headless_wait
echo [7/8] Headless three-process mode ready.
call :write_manifest headless
if errorlevel 1 goto :fail
if not defined OMOBA_HEADLESS_SECONDS set "OMOBA_HEADLESS_SECONDS=30"
powershell -NoProfile -Command "Start-Sleep -Seconds %OMOBA_HEADLESS_SECONDS%"
echo [8/8] Collecting headless result and stopping this run...
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dump_process_memory.ps1 -ProcessId !RUNTIME_PID_1! -ExpectedExe "%CLIENT_RUNTIME%" -OutputPath "%EVIDENCE_DIR%\team-1-runtime.dmp" -Role runtime
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dump_process_memory.ps1 -ProcessId !RUNTIME_PID_2! -ExpectedExe "%CLIENT_RUNTIME%" -OutputPath "%EVIDENCE_DIR%\team-2-runtime.dmp" -Role runtime
call :stop_runtimes
call :stop_backend
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\compare_fog_evidence.ps1 -EvidenceDir "%EVIDENCE_DIR%"
set "RUN_ERR=%errorlevel%"
popd
exit /b %RUN_ERR%

:write_manifest
set "MANIFEST_MODE=%~1"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\write_fog_run_manifest.ps1 -EvidenceDir "%EVIDENCE_DIR%" -ServerPid !BACKEND_PID! -Team1RuntimePid !RUNTIME_PID_1! -Team2RuntimePid !RUNTIME_PID_2! -Team1RendererPid !FRONTEND_PID_1! -Team2RendererPid !FRONTEND_PID_2! -ServerExe "%BACKEND%" -RuntimeExe "%CLIENT_RUNTIME%" -RendererExe "%EXECUTOR%" -Mode !MANIFEST_MODE!
exit /b %errorlevel%

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
set "PRESENTATION_ADDR=%~5"
set "FRONTEND_PID_FILE=omfx\target\frontend_%PLAYER_ID%.pid"
if exist "%FRONTEND_PID_FILE%" del "%FRONTEND_PID_FILE%" >nul 2>&1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\start_fog_demo_frontend.ps1 -Exe "%EXECUTOR%" -WorkingDirectory "omfx" -PidFile "%FRONTEND_PID_FILE%" -PlayerId %PLAYER_ID% -TeamId %TEAM_ID% -PlayerName "%PLAYER_NAME%" -WindowX %WINDOW_X% -PresentationAddr "%PRESENTATION_ADDR%"
if errorlevel 1 (
    echo Frontend %PLAYER_ID% start failed!
    exit /b 1
)
set /p FRONTEND_PID_%PLAYER_ID%=<"%FRONTEND_PID_FILE%"
echo   -^> frontend %PLAYER_ID% team %TEAM_ID% PID !FRONTEND_PID_%PLAYER_ID%!
exit /b 0

:start_runtime
set "PLAYER_ID=%~1"
set "TEAM_ID=%~2"
set "PLAYER_NAME=%~3"
set "PRESENTATION_ADDR=%~4"
set "RUNTIME_PID_FILE=omoba-client-runtime\target\team_%TEAM_ID%.pid"
if exist "%RUNTIME_PID_FILE%" del "%RUNTIME_PID_FILE%" >nul 2>&1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\start_client_runtime.ps1 -Exe "%CLIENT_RUNTIME%" -WorkingDirectory "." -PidFile "%RUNTIME_PID_FILE%" -PlayerId %PLAYER_ID% -TeamId %TEAM_ID% -PlayerName "%PLAYER_NAME%" -ServerAddr "%SERVER_ADDR%" -PresentationAddr "%PRESENTATION_ADDR%" -EvidenceDir "%EVIDENCE_DIR%"
if errorlevel 1 exit /b 1
set /p RUNTIME_PID_%TEAM_ID%=<"%RUNTIME_PID_FILE%"
echo   -^> runtime team %TEAM_ID% PID !RUNTIME_PID_%TEAM_ID%!
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
for %%P in (!FRONTEND_PID_1! !FRONTEND_PID_2!) do powershell -NoProfile -Command "$p=Get-Process -Id %%P -ErrorAction SilentlyContinue; if($p -and $p.Path -eq (Resolve-Path '%EXECUTOR%').Path){$null=$p.CloseMainWindow();if(-not$p.WaitForExit(2000)){Stop-Process -Id %%P -Force}}"
exit /b 0

:stop_runtimes
for %%P in (!RUNTIME_PID_1! !RUNTIME_PID_2!) do powershell -NoProfile -Command "$p=Get-Process -Id %%P -ErrorAction SilentlyContinue; if ($p -and $p.Path -eq (Resolve-Path '%CLIENT_RUNTIME%').Path) { Stop-Process -Id %%P -Force -ErrorAction SilentlyContinue }"
exit /b 0

:stop_backend
if defined BACKEND_PID (
    echo Stopping backend PID %BACKEND_PID%...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "$p=Get-Process -Id %BACKEND_PID% -ErrorAction SilentlyContinue;if($p -and $p.Path -eq (Resolve-Path '%BACKEND%').Path){Stop-Process -Id %BACKEND_PID% -Force}"
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
call :stop_frontends
call :stop_runtimes
call :stop_backend
popd
exit /b 1
