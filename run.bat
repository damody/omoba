@echo off
pushd %~dp0

echo [0/2] Killing stale processes (if any)...
taskkill /f /im omobab.exe >nul 2>&1
taskkill /f /im executor.exe >nul 2>&1

echo [1/2] Building backend (omb)...
cargo build --manifest-path omb\Cargo.toml
if %errorlevel% neq 0 (
    echo Backend build failed!
    popd
    pause
    exit /b 1
)

echo [2/2] Running frontend (spawns backend; backend dies when frontend exits)...
cargo run --manifest-path omfx\Cargo.toml -p executor

popd
