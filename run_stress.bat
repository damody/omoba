@echo off
REM ======================================================================
REM  run_stress.bat -- TD_STRESS perf test launcher (RELEASE build)
REM
REM  Steps:
REM    1. Kill stale omobab.exe / executor.exe
REM    2. Regenerate omb\Story\TD_STRESS\map.json
REM    3. Back up omb\game.toml, swap in omb\game_stress.toml
REM    4. Build base_content DLL (release) + omb backend (release).
REM       omfx hard-codes target\debug\omobab.exe, so we copy the release
REM       exe over target\debug\omobab.exe so omfx picks the fast build.
REM    5. Run omfx executor (release; spawns omb backend as a child)
REM    6. Always restore omb\game.toml afterwards
REM ======================================================================

pushd %~dp0

set TOML=omb\game.toml
set TOML_BAK=omb\game.toml.bak
set TOML_STRESS=omb\game_stress.toml

echo [0/5] Killing stale processes (if any)...
taskkill /f /im omobab.exe >nul 2>&1
taskkill /f /im executor.exe >nul 2>&1

echo [1/5] Regenerating stress map...
python scripts\gen_stress_map.py
if %errorlevel% neq 0 (
    echo   Stress map generation failed!
    popd
    pause
    exit /b 1
)

echo [2/5] Switching game.toml to stress variant (backup at %TOML_BAK%)...
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
echo [3/5] Building script DLL (scripts\base_content, release)...
cargo build --release --manifest-path scripts\Cargo.toml -p base_content
if %errorlevel% neq 0 (
    echo   Script DLL build failed!
    exit /b 1
)
if not exist omb\scripts mkdir omb\scripts
copy /y scripts\target\release\base_content.dll omb\scripts\base_content.dll >nul
echo   -^> copied base_content.dll (release) to omb\scripts\

echo [4/5] Building backend (omb, release)...
cargo build --release --manifest-path omb\Cargo.toml -p omobab
if %errorlevel% neq 0 (
    echo   Backend build failed!
    exit /b 1
)
REM omfx spawns target\debug\omobab.exe — copy release exe over it so the
REM perf test actually runs the optimized build.
if not exist omb\target\debug mkdir omb\target\debug
copy /y omb\target\release\omobab.exe omb\target\debug\omobab.exe >nul
echo   -^> copied omobab.exe (release) to omb\target\debug\ (for omfx spawn)

echo [5/5] Running frontend (omfx executor, release; spawns omb child)...
cargo run --release --manifest-path omfx\Cargo.toml -p executor
exit /b %errorlevel%

:restore
echo.
echo Restoring %TOML% from backup...
if exist "%TOML_BAK%" (
    copy /y "%TOML_BAK%" "%TOML%" >nul
    del "%TOML_BAK%" >nul 2>&1
)

popd
exit /b %MAIN_ERR%
