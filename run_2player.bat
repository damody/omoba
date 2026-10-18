@echo off
"%~dp0tools\lua\lua.exe" "%~dp0scripts\run_2player_interactive.lua" %*
exit /b %errorlevel%
