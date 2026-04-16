@echo off
pushd %~dp0

echo [1/2] Building backend (omb)...
cargo build --manifest-path omb\Cargo.toml
if %errorlevel% neq 0 (
    echo Backend build failed!
    popd
    pause
    exit /b 1
)

echo [2/2] Building and running frontend (omfx)...
cargo run --manifest-path omfx\Cargo.toml -p executor

popd
