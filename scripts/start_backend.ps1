param(
    [Parameter(Mandatory = $true)]
    [string]$Exe,

    [string]$WorkingDirectory = "omb",

    [string]$Stdout = "omb/log/launcher_backend_stdout.log",

    [string]$Stderr = "omb/log/launcher_backend_stderr.log"
)

$ErrorActionPreference = "Stop"

$exePath = (Resolve-Path -LiteralPath $Exe).Path
$workDir = (Resolve-Path -LiteralPath $WorkingDirectory).Path
$stdoutPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Stdout)
$stderrPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Stderr)

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $stdoutPath) | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $stderrPath) | Out-Null

$process = Start-Process `
    -FilePath $exePath `
    -WorkingDirectory $workDir `
    -RedirectStandardOutput $stdoutPath `
    -RedirectStandardError $stderrPath `
    -PassThru

$process.Id
