@echo off
pushd %~dp0

echo [0/3] Killing stale processes (if any)...
taskkill /f /im omobab.exe >nul 2>&1
taskkill /f /im executor.exe >nul 2>&1

echo [1/3] Building script DLL (scripts\base_content)...
cargo build --manifest-path scripts\Cargo.toml -p base_content
if %errorlevel% neq 0 (
    echo Script DLL build failed!
    popd
    pause
    exit /b 1
)
if not exist omb\scripts mkdir omb\scripts
copy /y scripts\target\debug\base_content.dll omb\scripts\base_content.dll >nul
echo   -^> copied base_content.dll to omb\scripts\

echo [2/3] Building backend (omb)...
cargo build --manifest-path omb\Cargo.toml
if %errorlevel% neq 0 (
    echo Backend build failed!
    popd
    pause
    exit /b 1
)

echo [3/3] Running frontend (spawns backend; backend dies when frontend exits)...
cargo run --manifest-path omfx\Cargo.toml -p executor

popd
