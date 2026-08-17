@echo off
setlocal

pushd "%~dp0.."
if errorlevel 1 exit /b 1

echo [1/2] Building release script DLL...
cargo build --manifest-path scripts\Cargo.toml -p base_content --release
set "RUN_ERR=%errorlevel%"
if not "%RUN_ERR%"=="0" goto :done

echo [2/2] Running TD rounds 1-100 autoplay test...
cargo test --manifest-path omoba-core\Cargo.toml --test td_autoplay_100 layered_td_coarse_autoplay_completes_rounds_1_to_100 -- --nocapture
set "RUN_ERR=%errorlevel%"

:done
popd
exit /b %RUN_ERR%
