@echo off
pushd %~dp0

echo [0/2] Killing stale omobab.exe instances (if any)...
taskkill /f /im omobab.exe >nul 2>&1

echo [1/2] Building backend (omb)...
cargo build --manifest-path omb\Cargo.toml
if %errorlevel% neq 0 (
    echo Backend build failed!
    popd
    pause
    exit /b 1
)

echo [2/2] Starting backend and frontend...
pushd omb
start "" /B target\debug\omobab.exe
popd

timeout /t 2 /nobreak >nul

cargo run --manifest-path omfx\Cargo.toml -p executor

echo Shutting down backend...
taskkill /f /im omobab.exe >nul 2>&1

popd
