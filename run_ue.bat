@echo off
"%~dp0tools\lua\lua.exe" "%~dp0scripts\run_ue.lua" %*
exit /b %errorlevel%
