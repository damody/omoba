@echo off
setlocal
set "OMB_NO_HEROES=1"
set "OMB_TD_STARTING_GOLD=10000"
call "%~dp0run.bat" %*
set "RUN_ERR=%errorlevel%"
exit /b %RUN_ERR%
