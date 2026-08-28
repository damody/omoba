param(
    [Parameter(Mandatory = $true)]
    [string]$Exe,

    [string]$WorkingDirectory = "omb",

    [string]$Stdout = "omb/log/launcher_backend_stdout.log",

    [string]$Stderr = "omb/log/launcher_backend_stderr.log",

    [string]$PidFile = ""
)

$ErrorActionPreference = "Stop"

function Normalize-ProcessPathEnv {
    $pathValue = [Environment]::GetEnvironmentVariable("Path", "Process")
    if ([string]::IsNullOrEmpty($pathValue)) {
        $pathValue = [Environment]::GetEnvironmentVariable("PATH", "Process")
    }
    if (-not [string]::IsNullOrEmpty($pathValue)) {
        [Environment]::SetEnvironmentVariable("PATH", $null, "Process")
        [Environment]::SetEnvironmentVariable("Path", $pathValue, "Process")
    }
}

Normalize-ProcessPathEnv

$exePath = (Resolve-Path -LiteralPath $Exe).Path
$workDir = (Resolve-Path -LiteralPath $WorkingDirectory).Path
$stdoutPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Stdout)
$stderrPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Stderr)
$pidFilePath = $null
if (-not [string]::IsNullOrWhiteSpace($PidFile)) {
    $pidFilePath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($PidFile)
}

function Convert-EnvPathToAbsolute([string]$Name) {
    $value = [Environment]::GetEnvironmentVariable($Name, "Process")
    if ([string]::IsNullOrWhiteSpace($value)) {
        return
    }
    if ([System.IO.Path]::IsPathRooted($value)) {
        return
    }
    $absolute = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($value)
    [Environment]::SetEnvironmentVariable($Name, $absolute, "Process")
}

Convert-EnvPathToAbsolute "OMB_GAME_TOML"
Convert-EnvPathToAbsolute "OMB_LUA_CONTENT_ROOT"
Convert-EnvPathToAbsolute "OMB_STORY_DATA_DIR"
Convert-EnvPathToAbsolute "OMB_SCENE_PATH"
Convert-EnvPathToAbsolute "OMB_SCRIPTS_DIR"
Convert-EnvPathToAbsolute "OMB_DLL_PATH"

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $stdoutPath) | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $stderrPath) | Out-Null
Remove-Item -LiteralPath $stdoutPath -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $stderrPath -Force -ErrorAction SilentlyContinue
if ($pidFilePath) {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $pidFilePath) | Out-Null
    Remove-Item -LiteralPath $pidFilePath -Force -ErrorAction SilentlyContinue
}

$process = Start-Process `
    -FilePath $exePath `
    -WorkingDirectory $workDir `
    -RedirectStandardOutput $stdoutPath `
    -RedirectStandardError $stderrPath `
    -WindowStyle Hidden `
    -PassThru

Start-Sleep -Milliseconds 1500
if ($process.HasExited) {
    if ($pidFilePath) {
        Remove-Item -LiteralPath $pidFilePath -Force -ErrorAction SilentlyContinue
    }
    throw "Backend process exited during startup with code $($process.ExitCode). See '$stdoutPath' and '$stderrPath'."
}
$deadline=[DateTime]::UtcNow.AddSeconds(20)
while([DateTime]::UtcNow-lt$deadline){
    if($process.HasExited){throw "Backend exited before KCP ready"}
    $logs=((Get-Content -Raw -LiteralPath $stdoutPath -ErrorAction SilentlyContinue)+(Get-Content -Raw -LiteralPath $stderrPath -ErrorAction SilentlyContinue))
    if($logs-match 'Starting KCP server on'){break}
    Start-Sleep -Milliseconds 100
}
if($logs-notmatch 'Starting KCP server on'){Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue;throw "Backend KCP ready timeout"}

if ($pidFilePath) {
    [System.IO.File]::WriteAllText($pidFilePath, "$($process.Id)`r`n", (New-Object System.Text.UTF8Encoding $false))
} else {
    $process.Id
}
