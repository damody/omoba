@echo off
echo [1/2] Building backend (omb)...
cd /d D:\omoba\omb
cargo build
if %errorlevel% neq 0 (
    echo Backend build failed!
    pause
    exit /b 1
)

echo [2/2] Building and running frontend (omfx)...
cd /d D:\omoba\omfx
cargo run -p executor
