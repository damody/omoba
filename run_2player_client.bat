@echo off
setlocal
pushd "%~dp0"

set "PLAYER_ID=%~1"
set "PLAYER_NAME=%~2"
set "LOCKSTEP_NAME=%~3"
set "TITLE_SUFFIX=%~4"
set "DONE_FILE=%~5"

set "OMB_PLAYER_ID=%PLAYER_ID%"
set "OMB_PLAYER_NAME=%PLAYER_NAME%"
set "OMB_LOCKSTEP_PLAYER_NAME=%LOCKSTEP_NAME%"
set "OMFX_LOG_SUFFIX=p%PLAYER_ID%"
set "OMFX_WINDOW_TITLE_SUFFIX=%TITLE_SUFFIX%"

echo Starting frontend %PLAYER_ID% (%PLAYER_NAME%)...
omfx\target\debug\executor.exe
set "RUN_ERR=%errorlevel%"

if defined DONE_FILE (
    >"%DONE_FILE%" echo %RUN_ERR%
)

popd
exit /b %RUN_ERR%
