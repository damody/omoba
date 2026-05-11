@echo off
setlocal

cd /d "%~dp0"

set WS_ADDR=127.0.0.1:50062
set KCP_ADDR=127.0.0.1:50061
set WEBROOT=omfx\executor-wasm\web-root
set MODE=%~1

echo [0/8] Killing stale web processes (if any)...
powershell -NoProfile -Command "Stop-Process -Name 'omobab','omb-ws-bridge','basic-http-server' -Force -ErrorAction SilentlyContinue"

echo [1/8] Checking Web toolchain...
where wasm-pack >nul 2>nul
if errorlevel 1 (
  echo wasm-pack is missing. Install it with: cargo install wasm-pack
  goto :fail_pause
)
rustup target add wasm32-unknown-unknown
if errorlevel 1 goto :fail_pause

echo [2/8] Building script DLL (scripts\base_content, debug)...
cargo build --manifest-path scripts\Cargo.toml -p base_content
if errorlevel 1 goto :fail_pause
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\dev_run_freshness.ps1 -Action stage-dll
if errorlevel 1 goto :fail_pause

echo [3/8] Building backend (omb, debug)...
cargo build --manifest-path omb\Cargo.toml -p omobab
if errorlevel 1 goto :fail_pause

echo [4/8] Building WebSocket bridge...
cargo build --manifest-path omb-ws-bridge\Cargo.toml
if errorlevel 1 goto :fail_pause

echo [5/8] Building Web/WASM executor...
pushd omfx\executor-wasm
wasm-pack build --target web --release
if errorlevel 1 goto :fail_popd
popd

echo [6/8] Staging web-root...
if not exist "%WEBROOT%" mkdir "%WEBROOT%"
if exist "%WEBROOT%\pkg" rmdir /s /q "%WEBROOT%\pkg"
xcopy /e /i /y "omfx\executor-wasm\pkg" "%WEBROOT%\pkg" >nul
if errorlevel 1 goto :fail_pause
copy /y "omfx\executor-wasm\index.html" "%WEBROOT%\index.html" >nul
if errorlevel 1 goto :fail_pause
copy /y "omfx\executor-wasm\main.js" "%WEBROOT%\main.js" >nul
if errorlevel 1 goto :fail_pause
if exist "omfx\data" (
  if exist "%WEBROOT%\data" rmdir /s /q "%WEBROOT%\data"
  xcopy /e /i /y "omfx\data" "%WEBROOT%\data" >nul
  if errorlevel 1 goto :fail_pause
)

if /i "%MODE%"=="--build-only" (
  echo Web build staged at %WEBROOT%
  exit /b 0
)

echo [7/8] Starting backend and WebSocket bridge...
start "omoba backend" cmd /k "cd /d "%~dp0omb" && target\debug\omobab.exe"
timeout /t 2 /nobreak >nul
start "omoba web bridge" cmd /k ""%~dp0omb-ws-bridge\target\debug\omb-ws-bridge.exe" %WS_ADDR% %KCP_ADDR%"

echo [8/8] Starting static web server...
where basic-http-server >nul 2>nul
if not errorlevel 1 (
  start "omoba web server" cmd /k "basic-http-server "%WEBROOT%""
) else (
  where py >nul 2>nul
  if errorlevel 1 (
    echo basic-http-server and py are both missing. Install one of them to serve %WEBROOT%.
    goto :fail_pause
  )
  start "omoba web server" cmd /k "py -3 -m http.server 4000 -d "%WEBROOT%""
)

echo.
echo Open this URL if the browser did not open automatically:
echo http://localhost:4000/?omoba_ws=ws://%WS_ADDR%^&player=web-player
start "" "http://localhost:4000/?omoba_ws=ws://%WS_ADDR%&player=web-player"
exit /b 0

:fail_popd
popd

:fail_pause
pause

:fail
exit /b 1
