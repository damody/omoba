@echo off
pushd %~dp0

echo [0/3] Killing stale processes (if any)...
taskkill /f /im omobab.exe >nul 2>&1
taskkill /f /im executor.exe >nul 2>&1

echo [1/3] Building backend (omb)...
cargo build --manifest-path omb\Cargo.toml
if %errorlevel% neq 0 (
    echo Backend build failed!
    popd
    pause
    exit /b 1
)

echo [2/3] Starting frontend (retries connecting every 1s until backend is up)...
start "omfx" cargo run --manifest-path omfx\Cargo.toml -p executor

echo [3/3] Starting backend...
pushd omb
target\debug\omobab.exe
popd

echo Shutting down frontend...
taskkill /f /im executor.exe >nul 2>&1

popd
