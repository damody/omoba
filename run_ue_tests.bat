@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "ROOT=%~dp0"
if "%ROOT:~-1%"=="\" set "ROOT=%ROOT:~0,-1%"
set "OMFUE=%ROOT%\omfue"
set "PROJECT=%OMFUE%\om.uproject"
set "TEST_FILTER=Om.Generated + Om.Playable + Om.Runtime"
set "CUSTOM_TEST="
set "SKIP_BUILD="

if defined UE_5_7_ROOT (
  set "UE_ROOT_RESOLVED=%UE_5_7_ROOT%"
) else if defined UE_ROOT (
  set "UE_ROOT_RESOLVED=%UE_ROOT%"
) else (
  set "UE_ROOT_RESOLVED=D:\UE5.7"
)

:parse_args
if "%~1"=="" goto :args_done
if /I "%~1"=="--no-build" (
  set "SKIP_BUILD=1"
  shift
  goto :parse_args
)
if /I "%~1"=="--test" (
  set "TEST_FILTER=%~2"
  set "CUSTOM_TEST=1"
  shift
  shift
  goto :parse_args
)
echo Unknown argument: %~1
exit /b 2

:args_done
set "UE_EDITOR_CMD=%UE_ROOT_RESOLVED%\Engine\Binaries\Win64\UnrealEditor-Cmd.exe"
set "REPORT_DIR=%OMFUE%\Saved\AutomationReports"

echo [run_ue_tests] root: %ROOT%
echo [run_ue_tests] UE root: %UE_ROOT_RESOLVED%
echo [run_ue_tests] project: %PROJECT%
echo [run_ue_tests] test filter: %TEST_FILTER%

if not exist "%PROJECT%" (
  echo [run_ue_tests] missing UE project: %PROJECT%
  exit /b 1
)
if not exist "%UE_EDITOR_CMD%" (
  echo [run_ue_tests] UnrealEditor-Cmd.exe not found under %UE_ROOT_RESOLVED%
  exit /b 1
)

if not defined SKIP_BUILD (
  call "%ROOT%\run_ue.bat" --build-only --single-player
  if errorlevel 1 exit /b 1
) else (
  echo [run_ue_tests] skipping build because --no-build was specified.
)

if not exist "%REPORT_DIR%" mkdir "%REPORT_DIR%"

if defined CUSTOM_TEST (
  call :run_filter "%TEST_FILTER%"
  if errorlevel 1 exit /b %errorlevel%
) else (
  call :run_filter "Om.Generated"
  if errorlevel 1 exit /b %errorlevel%
  call :run_filter "Om.Playable"
  if errorlevel 1 exit /b %errorlevel%
  call :run_filter "Om.Runtime"
  if errorlevel 1 exit /b %errorlevel%
)

echo [run_ue_tests] UE automation completed.
exit /b 0

:run_filter
set "CURRENT_FILTER=%~1"
set "CURRENT_REPORT_DIR=%REPORT_DIR%\%CURRENT_FILTER%"
if not exist "%CURRENT_REPORT_DIR%" mkdir "%CURRENT_REPORT_DIR%"
echo [run_ue_tests] running UE automation tests: %CURRENT_FILTER%
"%UE_EDITOR_CMD%" "%PROJECT%" -NullRHI -unattended -nop4 -nosplash -NoLoadingScreen -stdout -FullStdOutLogOutput -log -ExecCmds="Automation RunTests %CURRENT_FILTER%;Quit" -TestExit="Automation Test Queue Empty" -ReportOutputPath="%CURRENT_REPORT_DIR%"
set "TEST_EXIT=%errorlevel%"
if not "%TEST_EXIT%"=="0" (
  echo [run_ue_tests] UE automation failed for %CURRENT_FILTER% with exit code %TEST_EXIT%.
  exit /b %TEST_EXIT%
)
exit /b 0
