@echo off
REM ======================================================================
REM  run_stress.bat — TD_STRESS 性能壓測場景
REM
REM  流程：
REM   1. 清掉舊的 omobab.exe / executor.exe
REM   2. 重新生成 omb/Story/TD_STRESS/map.json（便於調參後馬上跑）
REM   3. 備份 omb/game.toml，暫時換成 omb/game_stress.toml
REM   4. build base_content DLL + omb backend（debug，因為 omfx 硬編碼
REM      找 target/debug/omobab.exe；要 release 請自行 cargo build --release
REM      並把 release exe 複製覆蓋 debug exe）
REM   5. 跑 omfx executor（會自動 spawn omb backend 讀切換後的 game.toml）
REM   6. 結束後 restore omb/game.toml
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

echo [2/5] Switching game.toml -^> stress variant (backup at %TOML_BAK%)...
if not exist "%TOML_STRESS%" (
    echo   %TOML_STRESS% missing!
    popd
    pause
    exit /b 1
)
copy /y "%TOML%" "%TOML_BAK%" >nul
copy /y "%TOML_STRESS%" "%TOML%" >nul

REM 跑完主流程後一律跳到 :restore，避免 game.toml 留在壓測版
call :main
set MAIN_ERR=%errorlevel%
goto :restore

:main
echo [3/5] Building script DLL (scripts\base_content)...
cargo build --manifest-path scripts\Cargo.toml -p base_content
if %errorlevel% neq 0 (
    echo   Script DLL build failed!
    exit /b 1
)
if not exist omb\scripts mkdir omb\scripts
copy /y scripts\target\debug\base_content.dll omb\scripts\base_content.dll >nul
echo   -^> copied base_content.dll to omb\scripts\

echo [4/5] Building backend (omb)...
cargo build --manifest-path omb\Cargo.toml
if %errorlevel% neq 0 (
    echo   Backend build failed!
    exit /b 1
)

echo [5/5] Running frontend (omfx spawns omb child process)...
cargo run --manifest-path omfx\Cargo.toml -p executor
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
