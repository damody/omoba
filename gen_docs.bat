@echo off
pushd %~dp0

echo [1/3] Building script DLL (scripts\base_content, release)...
cargo build --manifest-path scripts\Cargo.toml -p base_content --release
if %errorlevel% neq 0 (
    echo Script DLL build failed!
    popd
    pause
    exit /b 1
)
copy /y scripts\target\release\base_content.dll scripts\base_content.dll >nul
echo   -^> copied base_content.dll to scripts\

echo [2/3] Building gen-docs...
cargo build --manifest-path omb\Cargo.toml -p omobab --bin gen-docs --features gen-docs --release
if %errorlevel% neq 0 (
    echo gen-docs build failed!
    popd
    pause
    exit /b 1
)

echo [3/3] Generating catalog HTML...
pushd omb
cargo run -p omobab --bin gen-docs --features gen-docs --release
set GENDOCS_EXIT=%errorlevel%
popd
if %GENDOCS_EXIT% neq 0 (
    echo gen-docs run failed!
    popd
    pause
    exit /b 1
)

echo.
echo Done. Opening omb\target\docs\index.html ...
start "" "omb\target\docs\index.html"

popd
