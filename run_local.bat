@echo off
setlocal
pushd %~dp0

set ROOT=%~dp0

set OMB_DLL_PATH=%ROOT%omb\scripts\base_content.dll
set OMB_GAME_TOML=%ROOT%omb\game.toml
set OMB_STORY_DATA_DIR=%ROOT%scripts\lua_data

echo [run_local] OMB_DLL_PATH=%OMB_DLL_PATH%
echo [run_local] OMB_GAME_TOML=%OMB_GAME_TOML%
echo [run_local] OMB_STORY_DATA_DIR=%OMB_STORY_DATA_DIR%

call run.bat
popd
