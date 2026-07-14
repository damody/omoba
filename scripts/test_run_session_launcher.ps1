$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$launcherPath = Join-Path $repoRoot 'run.bat'
$bytes = [System.IO.File]::ReadAllBytes($launcherPath)
$text = [System.Text.Encoding]::UTF8.GetString($bytes)

if ($bytes.Length -ge 3 -and
    $bytes[0] -eq 0xEF -and
    $bytes[1] -eq 0xBB -and
    $bytes[2] -eq 0xBF) {
    throw 'run.bat must not contain a UTF-8 BOM'
}

if ($text -match "(?<!`r)`n") {
    throw 'run.bat must use CRLF line endings'
}

foreach ($forbidden in @(
    'call :start_backend',
    'call :stop_backend',
    ':start_backend',
    ':stop_backend'
)) {
    if ($text.Contains($forbidden)) {
        throw "run.bat must not manage an external backend: $forbidden"
    }
}

foreach ($required in @(
    'set "OMFX_BACKEND_EXE=%CD%\%BACKEND%"',
    'echo   -^> frontend session launcher will start backend: %OMFX_BACKEND_EXE%',
    '"%EXECUTOR%"'
)) {
    if (-not $text.Contains($required)) {
        throw "run.bat is missing required session-launcher wiring: $required"
    }
}

Write-Output 'run.bat session launcher verification passed'
