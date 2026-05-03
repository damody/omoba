@echo off
setlocal
pushd %~dp0
powershell -NoProfile -Command "Stop-Process -Name 'omobab','executor' -Force -ErrorAction SilentlyContinue"
cargo build --manifest-path scripts\Cargo.toml -p base_content >nul 2>&1
if not exist omb\scripts mkdir omb\scripts
copy /y scripts\target\debug\base_content.dll omb\scripts\base_content.dll >nul
cargo build --manifest-path omb\Cargo.toml >nul 2>&1
set OMFX_AUTO_START_AFTER_SEC=2
set OMFX_AUTO_EXIT_AFTER_SEC=60
cargo run --manifest-path omfx\Cargo.toml -p executor 2>&1
popd
