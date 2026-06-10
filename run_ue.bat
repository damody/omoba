@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "ROOT=%~dp0"
if "%ROOT:~-1%"=="\" set "ROOT=%ROOT:~0,-1%"
set "OMFUE=%ROOT%\omfue"
set "PROJECT=%OMFUE%\om.uproject"
for %%P in ("%PROJECT%") do set "UE_PROJECT_NAME=%%~nP"
set "UE_MAP=/Game/Map/Main"
set "RUN_MODE=game"
set "SMOKE_SECONDS=90"
set "SKIP_BUILD=1"
set "UE_RHI_ARG=-d3d11"
set "RUN_BACKEND=0"
set "UE_RUNTIME_ARG=-om-single-player"
if not defined CARGO_INCREMENTAL set "CARGO_INCREMENTAL=1"

if defined UE_5_7_ROOT (
  set "UE_ROOT_RESOLVED=%UE_5_7_ROOT%"
  set "UE_ROOT_SOURCE=UE_5_7_ROOT"
) else if defined UE_ROOT (
  set "UE_ROOT_RESOLVED=%UE_ROOT%"
  set "UE_ROOT_SOURCE=UE_ROOT"
) else if exist "D:\UE_5.7\Engine\Binaries\Win64\UnrealEditor.exe" (
  set "UE_ROOT_RESOLVED=D:\UE_5.7"
  set "UE_ROOT_SOURCE=default D:\UE_5.7"
) else if exist "D:\UE5.7\Engine\Binaries\Win64\UnrealEditor.exe" (
  set "UE_ROOT_RESOLVED=D:\UE5.7"
  set "UE_ROOT_SOURCE=default D:\UE5.7"
) else if exist "C:\Program Files\Epic Games\UE_5.7\Engine\Binaries\Win64\UnrealEditor.exe" (
  set "UE_ROOT_RESOLVED=C:\Program Files\Epic Games\UE_5.7"
  set "UE_ROOT_SOURCE=default Epic Games UE_5.7"
) else (
  set "UE_ROOT_RESOLVED=D:\UE_5.7"
  set "UE_ROOT_SOURCE=default missing"
)

:parse_args
if "%~1"=="" goto :args_done
if /I "%~1"=="--editor" (
  set "RUN_MODE=editor"
  shift
  goto :parse_args
)
if /I "%~1"=="--headless-smoke" (
  set "RUN_MODE=headless"
  shift
  goto :parse_args
)
if /I "%~1"=="--build-only" (
  set "RUN_MODE=build-only"
  set "SKIP_BUILD="
  shift
  goto :parse_args
)
if /I "%~1"=="--build" (
  set "SKIP_BUILD="
  shift
  goto :parse_args
)
if /I "%~1"=="--game-smoke" (
  set "RUN_MODE=game-smoke"
  shift
  goto :parse_args
)
if /I "%~1"=="--safe" (
  set "RUN_MODE=safe"
  shift
  goto :parse_args
)
if /I "%~1"=="--dx12" (
  set "UE_RHI_ARG=-d3d12"
  shift
  goto :parse_args
)
if /I "%~1"=="--d3d11" (
  set "UE_RHI_ARG=-d3d11"
  shift
  goto :parse_args
)
if /I "%~1"=="--networked" (
  set "RUN_BACKEND=1"
  set "UE_RUNTIME_ARG=-om-networked"
  shift
  goto :parse_args
)
if /I "%~1"=="--with-backend" (
  set "RUN_BACKEND=1"
  set "UE_RUNTIME_ARG=-om-networked"
  shift
  goto :parse_args
)
if /I "%~1"=="--single-player" (
  set "RUN_BACKEND=0"
  set "UE_RUNTIME_ARG=-om-single-player"
  shift
  goto :parse_args
)
if /I "%~1"=="--seconds" (
  set "SMOKE_SECONDS=%~2"
  shift
  shift
  goto :parse_args
)
if /I "%~1"=="--no-build" (
  set "SKIP_BUILD=1"
  shift
  goto :parse_args
)
echo Unknown argument: %~1
exit /b 2

:args_done
set "UE_EDITOR=%UE_ROOT_RESOLVED%\Engine\Binaries\Win64\UnrealEditor.exe"
set "UE_EDITOR_CMD=%UE_ROOT_RESOLVED%\Engine\Binaries\Win64\UnrealEditor-Cmd.exe"
set "UE_BUILD=%UE_ROOT_RESOLVED%\Engine\Build\BatchFiles\Build.bat"
set "UE_UAT=%UE_ROOT_RESOLVED%\Engine\Build\BatchFiles\RunUAT.bat"
set "UE_GAME_EXE=%OMFUE%\Binaries\Win64\OmGame.exe"
set "UE_STAGED_DIR=%OMFUE%\Saved\StagedBuilds\Windows"
set "UE_STAGED_EXE=%UE_STAGED_DIR%\%UE_PROJECT_NAME%\Binaries\Win64\OmGame.exe"
set "UE_STAGED_BOOTSTRAP_EXE=%UE_STAGED_DIR%\OmGame.exe"
set "BACKEND_EXE=%ROOT%\omb\target\debug\omobab.exe"
set "BASE_CONTENT_TARGET=%ROOT%\scripts\target\debug\base_content.dll"
set "BASE_CONTENT_STAGED=%ROOT%\scripts\base_content.dll"
set "BACKEND_PID_FILE=%ROOT%\omb\log\run_ue_backend.pid"
set "BACKEND_STDOUT=%ROOT%\omb\log\run_ue_backend_stdout.log"
set "BACKEND_STDERR=%ROOT%\omb\log\run_ue_backend_stderr.log"
set "UE_STDOUT=%OMFUE%\Saved\Logs\run_ue_stdout.log"
set "UE_STDERR=%OMFUE%\Saved\Logs\run_ue_stderr.log"

set "OMB_GAME_TOML=%ROOT%\omb\game.toml"
set "OMB_LUA_CONTENT=1"
set "OMB_LUA_CONTENT_ROOT=%ROOT%\scripts\lua_data"
set "OMB_STORY_DATA_DIR=%ROOT%\scripts\lua_data"
set "OMB_SCRIPTS_DIR=%ROOT%\scripts"
set "OMB_DLL_PATH=%BASE_CONTENT_STAGED%"
set "OMB_STORY=TD_1"

echo [run_ue] root: %ROOT%
echo [run_ue] UE root: %UE_ROOT_RESOLVED% (%UE_ROOT_SOURCE%)
echo [run_ue] project: %PROJECT%
echo [run_ue] mode: %RUN_MODE%
if "%RUN_BACKEND%"=="1" (
  echo [run_ue] runtime: networked backend
) else (
  echo [run_ue] runtime: single-player local simulation
)

if not exist "%PROJECT%" (
  echo [run_ue] missing UE project: %PROJECT%
  goto :fail
)
if not exist "%UE_EDITOR%" (
  echo [run_ue] UnrealEditor.exe not found under %UE_ROOT_RESOLVED%
  goto :fail
)
if not exist "%UE_BUILD%" (
  echo [run_ue] Build.bat not found under %UE_ROOT_RESOLVED%
  goto :fail
)
if not exist "%UE_UAT%" (
  echo [run_ue] RunUAT.bat not found under %UE_ROOT_RESOLVED%
  goto :fail
)
if not defined SKIP_BUILD (
  call :build_all
  if errorlevel 1 goto :fail
) else (
  echo [run_ue] skipping build. Use --build or --build-only to rebuild.
)

if /I "%RUN_MODE%"=="build-only" (
  echo [run_ue] build-only completed.
  exit /b 0
)

if "%RUN_BACKEND%"=="1" (
  call :start_backend
  if errorlevel 1 goto :fail
) else (
  echo [run_ue] backend not started because single-player runtime is enabled.
)

call :run_frontend
set "RUN_ERR=%errorlevel%"

call :stop_backend
exit /b %RUN_ERR%

:build_all
pushd "%ROOT%" >nul

where cargo >nul 2>&1
if errorlevel 1 (
  echo [run_ue] cargo was not found in PATH.
  popd >nul
  exit /b 1
)
where cbindgen >nul 2>&1
if errorlevel 1 (
  echo [run_ue] cbindgen was not found in PATH.
  popd >nul
  exit /b 1
)

echo [run_ue] [1/5] incrementally building script DLL with runtime Lua content...
cargo build --manifest-path "%ROOT%\scripts\Cargo.toml" -p base_content --features runtime-lua-content
if errorlevel 1 (
  popd >nul
  exit /b 1
)

if not exist "%BASE_CONTENT_TARGET%" (
  echo [run_ue] missing built script DLL: %BASE_CONTENT_TARGET%
  popd >nul
  exit /b 1
)
call :copy_if_changed "%BASE_CONTENT_TARGET%" "%BASE_CONTENT_STAGED%" "scripts\base_content.dll"
if errorlevel 1 (
  popd >nul
  exit /b 1
)
if not exist "%ROOT%\omb\scripts" mkdir "%ROOT%\omb\scripts"
call :copy_if_changed "%BASE_CONTENT_TARGET%" "%ROOT%\omb\scripts\base_content.dll" "omb\scripts\base_content.dll"
if errorlevel 1 (
  popd >nul
  exit /b 1
)

if "%RUN_BACKEND%"=="1" (
  echo [run_ue] [2/5] incrementally building Rust backend with runtime Lua content...
  cargo build --manifest-path "%ROOT%\omb\Cargo.toml" -p omobab --features runtime-lua-content
  if errorlevel 1 (
    popd >nul
    exit /b 1
  )
) else (
  echo [run_ue] [2/5] skipping Rust backend build for single-player runtime.
)

echo [run_ue] [3/5] incrementally generating and staging UE bridge...
call "%OMFUE%\build_bridge.bat"
if errorlevel 1 (
  popd >nul
  exit /b 1
)

echo [run_ue] [4/5] checking generated bridge/code freshness...
call "%OMFUE%\check_om_fresh.bat"
if errorlevel 1 (
  popd >nul
  exit /b 1
)

if /I "%RUN_MODE%"=="editor" goto :build_ue_editor
if /I "%RUN_MODE%"=="headless" goto :build_ue_editor
if /I "%RUN_MODE%"=="safe" goto :build_ue_editor
if /I "%RUN_MODE%"=="build-only" goto :build_ue_editor

echo [run_ue] [5/5] building, cooking, and staging UE standalone game incrementally...
call "%UE_UAT%" BuildCookRun -project="%PROJECT%" -noP4 -platform=Win64 -clientconfig=Development -build -cook -stage -iterativecooking -cookincremental -nocleanstage -map="%UE_MAP%" -NoCompileEditor -unattended -utf8output
if errorlevel 1 (
  popd >nul
  exit /b 1
)
call :resolve_staged_exe
if errorlevel 1 (
  echo [run_ue] missing staged UE game executable under %OMFUE%\Saved\StagedBuilds
  popd >nul
  exit /b 1
)
call :stage_runtime_script_dll
if errorlevel 1 (
  popd >nul
  exit /b 1
)
echo [run_ue] staged UE game: %UE_RUN_EXE%
popd >nul
exit /b 0

:build_ue_editor
echo [run_ue] [5/5] incrementally building UE editor target...
call "%UE_BUILD%" OmGameEditor Win64 Development -Project="%PROJECT%" -WaitMutex -NoHotReloadFromIDE
if errorlevel 1 (
  popd >nul
  exit /b 1
)

popd >nul
exit /b 0

:start_backend
echo [run_ue] starting Rust backend...
if not exist "%ROOT%\omb\log" mkdir "%ROOT%\omb\log"
if exist "%BACKEND_PID_FILE%" (
  for /f "usebackq delims=" %%P in ("%BACKEND_PID_FILE%") do powershell -NoProfile -ExecutionPolicy Bypass -Command "Stop-Process -Id %%P -Force -ErrorAction SilentlyContinue"
  del "%BACKEND_PID_FILE%" >nul 2>&1
)
taskkill /f /im omobab.exe >nul 2>&1

powershell -NoProfile -ExecutionPolicy Bypass -File "%ROOT%\scripts\start_backend.ps1" -Exe "%BACKEND_EXE%" -WorkingDirectory "%ROOT%\omb" -Stdout "%BACKEND_STDOUT%" -Stderr "%BACKEND_STDERR%" -PidFile "%BACKEND_PID_FILE%"
if errorlevel 1 (
  echo [run_ue] backend failed to start.
  if exist "%BACKEND_STDOUT%" type "%BACKEND_STDOUT%"
  if exist "%BACKEND_STDERR%" type "%BACKEND_STDERR%"
  exit /b 1
)

set "BACKEND_PID="
if exist "%BACKEND_PID_FILE%" set /p BACKEND_PID=<"%BACKEND_PID_FILE%"
if not defined BACKEND_PID (
  echo [run_ue] backend pid file was not written: %BACKEND_PID_FILE%
  exit /b 1
)
echo [run_ue] backend PID: %BACKEND_PID%
exit /b 0

:run_frontend
if /I "%RUN_MODE%"=="editor" (
  echo [run_ue] launching UE editor. Press Play to run the frontend world.
  "%UE_EDITOR%" "%PROJECT%" %UE_RUNTIME_ARG%
  exit /b !errorlevel!
)

if /I "%RUN_MODE%"=="headless" (
  echo [run_ue] launching headless UE frontend smoke for %SMOKE_SECONDS%s...
  if not exist "%OMFUE%\Saved\Logs" mkdir "%OMFUE%\Saved\Logs"
  if exist "%OMFUE%\Saved\Logs\om.log" del "%OMFUE%\Saved\Logs\om.log" >nul 2>&1
  if exist "%UE_STDOUT%" del "%UE_STDOUT%" >nul 2>&1
  if exist "%UE_STDERR%" del "%UE_STDERR%" >nul 2>&1
  powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $p = Start-Process -FilePath '%UE_EDITOR_CMD%' -ArgumentList @('%PROJECT%', '%UE_MAP%', '-game', '%UE_RUNTIME_ARG%', '-NullRHI', '-unattended', '-nop4', '-nosplash', '-stdout', '-FullStdOutLogOutput', '-log') -RedirectStandardOutput '%UE_STDOUT%' -RedirectStandardError '%UE_STDERR%' -WindowStyle Hidden -PassThru; Start-Sleep -Seconds %SMOKE_SECONDS%; if ($p.HasExited) { exit $p.ExitCode } else { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue; exit 0 }"
  set "FRONTEND_ERR=%errorlevel%"
  if not "!FRONTEND_ERR!"=="0" exit /b !FRONTEND_ERR!
  call :assert_runtime_started
  exit /b !errorlevel!
)

if /I "%RUN_MODE%"=="safe" (
  echo [run_ue] launching safe UE frontend smoke with NullRHI for %SMOKE_SECONDS%s...
  if not exist "%OMFUE%\Saved\Logs" mkdir "%OMFUE%\Saved\Logs"
  if exist "%OMFUE%\Saved\Logs\om.log" del "%OMFUE%\Saved\Logs\om.log" >nul 2>&1
  if exist "%UE_STDOUT%" del "%UE_STDOUT%" >nul 2>&1
  if exist "%UE_STDERR%" del "%UE_STDERR%" >nul 2>&1
  powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $p = Start-Process -FilePath '%UE_EDITOR%' -ArgumentList @('%PROJECT%', '%UE_MAP%', '-game', '%UE_RUNTIME_ARG%', '-NullRHI', '-unattended', '-nop4', '-nosplash', '-stdout', '-FullStdOutLogOutput', '-log') -RedirectStandardOutput '%UE_STDOUT%' -RedirectStandardError '%UE_STDERR%' -WindowStyle Hidden -PassThru; Start-Sleep -Seconds %SMOKE_SECONDS%; if ($p.HasExited) { exit $p.ExitCode } else { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue; exit 0 }"
  set "FRONTEND_ERR=%errorlevel%"
  if not "!FRONTEND_ERR!"=="0" exit /b !FRONTEND_ERR!
  call :assert_runtime_started
  exit /b !errorlevel!
)

if /I "%RUN_MODE%"=="game-smoke" (
  echo [run_ue] launching bounded standalone UE frontend smoke for %SMOKE_SECONDS%s with %UE_RHI_ARG%...
  if not exist "%OMFUE%\Saved\Logs" mkdir "%OMFUE%\Saved\Logs"
  call :resolve_staged_exe
  if errorlevel 1 (
    echo [run_ue] missing staged UE game executable. Run with --build once to cook and stage it.
    exit /b 1
  )
  call :stage_runtime_script_dll
  if errorlevel 1 exit /b 1
  if exist "%OMFUE%\Saved\Logs\om.log" del "%OMFUE%\Saved\Logs\om.log" >nul 2>&1
  if exist "%UE_STDOUT%" del "%UE_STDOUT%" >nul 2>&1
  if exist "%UE_STDERR%" del "%UE_STDERR%" >nul 2>&1
  for %%D in ("!UE_RUN_EXE!") do set "UE_RUN_DIR=%%~dpD"
  powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $p = Start-Process -FilePath '!UE_RUN_EXE!' -WorkingDirectory '!UE_RUN_DIR!' -ArgumentList @('%UE_MAP%', '%UE_RHI_ARG%', '%UE_RUNTIME_ARG%', '-noraytracing', '-windowed', '-ResX=1280', '-ResY=720', '-unattended', '-nop4', '-nosplash', '-stdout', '-FullStdOutLogOutput', '-log') -RedirectStandardOutput '%UE_STDOUT%' -RedirectStandardError '%UE_STDERR%' -PassThru; Start-Sleep -Seconds %SMOKE_SECONDS%; if ($p.HasExited) { exit $p.ExitCode } else { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue; exit 0 }"
  set "FRONTEND_ERR=%errorlevel%"
  if not "!FRONTEND_ERR!"=="0" exit /b !FRONTEND_ERR!
  call :assert_runtime_started
  exit /b !errorlevel!
)

call :resolve_staged_exe
if errorlevel 1 (
  echo [run_ue] missing staged UE game executable. Run with --build once to cook and stage it.
  exit /b 1
)
call :stage_runtime_script_dll
if errorlevel 1 exit /b 1
echo [run_ue] launching UE standalone game frontend...
for %%D in ("%UE_RUN_EXE%") do set "UE_RUN_DIR=%%~dpD"
pushd "%UE_RUN_DIR%" >nul
"%UE_RUN_EXE%" "%UE_MAP%" %UE_RHI_ARG% %UE_RUNTIME_ARG% -noraytracing -windowed -ResX=1280 -ResY=720 -unattended -nop4 -nosplash -stdout -FullStdOutLogOutput -log
set "UE_EXIT=%errorlevel%"
popd >nul
exit /b %UE_EXIT%

:resolve_staged_exe
set "UE_RUN_EXE="
set "UE_STAGE_ROOT="
if exist "%UE_STAGED_EXE%" (
  set "UE_RUN_EXE=%UE_STAGED_EXE%"
  set "UE_STAGE_ROOT=%UE_STAGED_DIR%"
  exit /b 0
)
if exist "%UE_STAGED_BOOTSTRAP_EXE%" (
  set "UE_RUN_EXE=%UE_STAGED_BOOTSTRAP_EXE%"
  set "UE_STAGE_ROOT=%UE_STAGED_DIR%"
  exit /b 0
)
for %%F in ("%OMFUE%\Saved\StagedBuilds\Windows\*.exe" "%OMFUE%\Saved\StagedBuilds\WindowsNoEditor\*.exe" "%OMFUE%\Saved\StagedBuilds\Win64\*.exe") do (
  if exist "%%~fF" (
    set "UE_RUN_EXE=%%~fF"
    for %%D in ("%%~fF") do set "UE_STAGE_ROOT=%%~dpD"
    if "!UE_STAGE_ROOT:~-1!"=="\" set "UE_STAGE_ROOT=!UE_STAGE_ROOT:~0,-1!"
    exit /b 0
  )
)
exit /b 1

:stage_runtime_script_dll
if not defined UE_RUN_EXE (
  echo [run_ue] cannot stage script DLL before resolving UE_RUN_EXE.
  exit /b 1
)
if not defined UE_STAGE_ROOT set "UE_STAGE_ROOT=%UE_STAGED_DIR%"
set "STAGED_OM_PLUGIN_BIN=%UE_STAGE_ROOT%\%UE_PROJECT_NAME%\Plugins\OmRuntime\Binaries\Win64"
set "SOURCE_BASE_CONTENT=%OMFUE%\Plugins\OmRuntime\Binaries\Win64\base_content.dll"
if not exist "%SOURCE_BASE_CONTENT%" (
  echo [run_ue] missing source script DLL for staging: %SOURCE_BASE_CONTENT%
  exit /b 1
)
if not exist "%STAGED_OM_PLUGIN_BIN%" mkdir "%STAGED_OM_PLUGIN_BIN%"
call :copy_if_changed "%SOURCE_BASE_CONTENT%" "%STAGED_OM_PLUGIN_BIN%\base_content.dll" "staged base_content.dll"
if errorlevel 1 exit /b %errorlevel%
if not exist "%STAGED_OM_PLUGIN_BIN%\base_content.dll" (
  echo [run_ue] failed to stage script DLL to %STAGED_OM_PLUGIN_BIN%\base_content.dll
  exit /b 1
)
exit /b 0

:assert_runtime_started
set "RUNTIME_MARKER_FOUND="
if exist "%OMFUE%\Saved\Logs\om.log" (
  findstr /C:"Started bridge runtime" "%OMFUE%\Saved\Logs\om.log" >nul 2>&1
  if not errorlevel 1 set "RUNTIME_MARKER_FOUND=1"
)
if not defined RUNTIME_MARKER_FOUND if exist "%UE_STDOUT%" (
  findstr /C:"Started bridge runtime" "%UE_STDOUT%" >nul 2>&1
  if not errorlevel 1 set "RUNTIME_MARKER_FOUND=1"
)
if not defined RUNTIME_MARKER_FOUND (
  echo [run_ue] UE smoke did not reach bridge runtime startup. Recent UE log:
  if exist "%OMFUE%\Saved\Logs\om.log" powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-Content -Tail 80 '%OMFUE%\Saved\Logs\om.log'"
  if exist "%UE_STDOUT%" powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-Content -Tail 80 '%UE_STDOUT%'"
  exit /b 1
)
echo [run_ue] UE smoke reached bridge runtime startup.
exit /b 0

:copy_if_changed
set "COPY_SRC=%~1"
set "COPY_DST=%~2"
set "COPY_LABEL=%~3"
if not exist "%COPY_SRC%" (
  echo [run_ue] missing source for %COPY_LABEL%: %COPY_SRC%
  exit /b 1
)
if exist "%COPY_DST%" (
  fc /b "%COPY_SRC%" "%COPY_DST%" >nul
  if not errorlevel 1 (
    echo [run_ue] unchanged: %COPY_LABEL%
    exit /b 0
  )
)
copy /Y "%COPY_SRC%" "%COPY_DST%" >nul
if errorlevel 1 (
  echo [run_ue] failed to stage %COPY_LABEL% to %COPY_DST%
  exit /b 1
)
echo [run_ue] updated: %COPY_LABEL%
exit /b 0

:stop_backend
if defined BACKEND_PID (
  echo [run_ue] stopping backend PID %BACKEND_PID%...
  powershell -NoProfile -ExecutionPolicy Bypass -Command "Stop-Process -Id %BACKEND_PID% -Force -ErrorAction SilentlyContinue"
)
if exist "%BACKEND_PID_FILE%" del "%BACKEND_PID_FILE%" >nul 2>&1
set "BACKEND_PID="
exit /b 0

:fail
call :stop_backend
exit /b 1
