@echo off
setlocal
set "OMFX_SANDBOX=1"
pushd "%~dp0"

REM Options:
REM   --trace  啟用 omfx Perfetto trace（輸出預設由 executor 決定；可先設定
REM            OMFX_PERFETTO_PATH / OMFX_PERFETTO_DETAIL / OMFX_PERFETTO_MAX_SECONDS）

set "FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1"
set "EXECUTOR=omfx\target\debug\executor.exe"
set "BACKEND=omb\target\debug\omobab.exe"
set "OMFX_BACKEND_EXE=%CD%\%BACKEND%"
set "OMB_GAME_TOML=%CD%\omb\game.toml"
set "OMFX_GAME_TOML=%CD%\omfx\game.toml"
set "OMB_STORY=TD_1"
set "OMB_SCENE_PATH="
set "CARGO_PROFILE_DEV_DEBUG=false"
set "CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=false"
set "RUSTFLAGS=-C debuginfo=0"

set "RUN_TRACE="
:parse_args
if "%~1"=="" goto :args_done
if /I "%~1"=="--trace" set "RUN_TRACE=1"
shift
goto :parse_args

:args_done
if defined RUN_TRACE (
    set "OMFX_PERFETTO_TRACE=1"
    if not defined OMFX_PERFETTO_DETAIL set "OMFX_PERFETTO_DETAIL=frame"
    if not defined OMFX_PERFETTO_PATH set "OMFX_PERFETTO_PATH=omfx\target\profiles\run.perfetto-trace"
    echo Perfetto trace enabled for run.
    echo   -^> trace path: %OMFX_PERFETTO_PATH%
)

echo [0/5] Killing stale processes (if any)...
taskkill /f /im omobab.exe >nul 2>&1
taskkill /f /im executor.exe >nul 2>&1

echo [1/5] Checking script DLL (scripts\base_content)...
call :ensure_fresh script-dll "script DLL"
if errorlevel 1 goto :fail

%FRESHNESS% -Action stage-dll
if errorlevel 1 (
    echo Script DLL staging failed!
    goto :fail_pause
)

echo [2/5] Checking backend (omb)...
call :ensure_fresh backend "backend"
if errorlevel 1 goto :fail

echo [3/5] Checking frontend (omfx executor)...
call :ensure_fresh frontend "frontend"
if errorlevel 1 goto :fail

if not exist "%BACKEND%" (
    echo Backend executable missing: %BACKEND%
    goto :fail_pause
)

if not exist "%EXECUTOR%" (
    echo Frontend executable missing: %EXECUTOR%
    goto :fail_pause
)

echo [4/4] Running frontend...
echo   -^> frontend session launcher will start backend: %OMFX_BACKEND_EXE%
"%EXECUTOR%"
set "RUN_ERR=%errorlevel%"
popd
exit /b %RUN_ERR%

:ensure_fresh
set "ARTIFACT=%~1"
set "LABEL=%~2"

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

if "%ARTIFACT%"=="script-dll" (
    cargo build --manifest-path scripts\Cargo.toml -p base_content --features runtime-lua-content
    if errorlevel 1 (
        echo Script DLL build failed!
        pause
        exit /b 1
    )
    exit /b 0
)

if "%ARTIFACT%"=="backend" (
    cargo build --manifest-path omb\Cargo.toml --features runtime-lua-content
    if errorlevel 1 (
        echo Backend build failed!
        pause
        exit /b 1
    )
    exit /b 0
)

if "%ARTIFACT%"=="frontend" (
    cargo build --manifest-path omfx\Cargo.toml -p executor --features runtime-lua-content
    if errorlevel 1 (
        echo Frontend build failed!
        pause
        exit /b 1
    )
    exit /b 0
)

echo Unknown freshness artifact: %ARTIFACT%
exit /b 1

:fail_pause
pause

:fail
popd
exit /b 1
