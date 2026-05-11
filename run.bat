@echo off
setlocal
pushd %~dp0

set FRESHNESS=powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1
set EXECUTOR=omfx\target\debug\executor.exe
set OMB_LUA_CONTENT=1
set OMB_LUA_CONTENT_ROOT=%CD%\scripts\lua_data
set OMB_STORY_DATA_DIR=%OMB_LUA_CONTENT_ROOT%

echo [0/4] Killing stale processes (if any)...
taskkill /f /im omobab.exe >nul 2>&1
taskkill /f /im executor.exe >nul 2>&1

echo [1/4] Checking script DLL (scripts\base_content)...
call :ensure_fresh script-dll "script DLL" "cargo build --manifest-path scripts\Cargo.toml -p base_content --features runtime-lua-content" "Script DLL build failed!"
if errorlevel 1 goto :fail

%FRESHNESS% -Action stage-dll
if errorlevel 1 (
    echo Script DLL staging failed!
    goto :fail_pause
)

echo [2/4] Checking backend (omb)...
call :ensure_fresh backend "backend" "cargo build --manifest-path omb\Cargo.toml --features runtime-lua-content" "Backend build failed!"
if errorlevel 1 goto :fail

echo [3/4] Checking frontend (omfx executor)...
call :ensure_fresh frontend "frontend" "cargo build --manifest-path omfx\Cargo.toml -p executor --features runtime-lua-content" "Frontend build failed!"
if errorlevel 1 goto :fail

if not exist "%EXECUTOR%" (
    echo Frontend executable missing: %EXECUTOR%
    goto :fail_pause
)

echo [4/4] Running frontend (spawns backend; backend dies when frontend exits)...
"%EXECUTOR%"
set RUN_ERR=%errorlevel%
popd
exit /b %RUN_ERR%

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
popd
exit /b 1
