@echo off
"%~dp0tools\lua\lua.exe" "%~dp0scripts\run.lua" %*
exit /b %errorlevel%
