@echo off
"%~dp0tools\lua\lua.exe" "%~dp0scripts\run_10000.lua" %*
exit /b %errorlevel%
