param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$WorkingDirectory,
    [Parameter(Mandatory = $true)][string]$PidFile,
    [Parameter(Mandatory = $true)][int]$PlayerId,
    [Parameter(Mandatory = $true)][int]$TeamId,
    [Parameter(Mandatory = $true)][string]$PlayerName,
    [Parameter(Mandatory = $true)][string]$ServerAddr,
    [Parameter(Mandatory = $true)][string]$PresentationAddr,
    [string]$EvidenceDir = ""
)

$ErrorActionPreference = 'Stop'
$resolvedExe = (Resolve-Path -LiteralPath $Exe).Path
$resolvedWorkingDirectory = (Resolve-Path -LiteralPath $WorkingDirectory).Path
$resolvedPidFile = [System.IO.Path]::GetFullPath($PidFile)
[System.IO.Directory]::CreateDirectory([System.IO.Path]::GetDirectoryName($resolvedPidFile)) | Out-Null
$logDir = Join-Path ([System.IO.Path]::GetDirectoryName($resolvedPidFile)) "runtime-logs"
[System.IO.Directory]::CreateDirectory($logDir) | Out-Null
$stdout = Join-Path $logDir "team-$TeamId.stdout.log"
$stderr = Join-Path $logDir "team-$TeamId.stderr.log"
$arguments = @(
    '--player-id', [string]$PlayerId,
    '--team', [string]$TeamId,
    '--player-name', $PlayerName,
    '--server', $ServerAddr,
    '--presentation-bind', $PresentationAddr,
    '--presentation-hz', '60',
    '--protocol-version', '2',
    '--scripted-move-tick', '300',
    '--scripted-hidden-target-tick', '420',
    '--screenshot-tick', '600'
)
if (-not [string]::IsNullOrWhiteSpace($EvidenceDir)) {
    $arguments += @('--test-mode', '--evidence-dir', [System.IO.Path]::GetFullPath($EvidenceDir))
    if ($TeamId -eq 1 -and $env:OMOBA_TEAM1_FAULT_TICK) { $arguments += @('--fault-tick', $env:OMOBA_TEAM1_FAULT_TICK) }
}
$process = Start-Process -FilePath $resolvedExe -WorkingDirectory $resolvedWorkingDirectory `
    -ArgumentList $arguments -RedirectStandardOutput $stdout -RedirectStandardError $stderr `
    -WindowStyle Hidden -PassThru
Start-Sleep -Milliseconds 500
if ($process.HasExited) {
    throw "Team $TeamId runtime exited during startup. See $stderr"
}
$deadline=[DateTime]::UtcNow.AddSeconds(20)
while([DateTime]::UtcNow-lt$deadline){
    if($process.HasExited){throw "Team $TeamId runtime exited before ready. See $stderr"}
    $readyManifest=if([string]::IsNullOrWhiteSpace($EvidenceDir)){$null}else{Join-Path ([IO.Path]::GetFullPath($EvidenceDir)) "team-$TeamId-runtime\manifest.json"}
    if(($readyManifest-and(Test-Path -LiteralPath $readyManifest))-or((Test-Path -LiteralPath $stderr)-and((Get-Content -Raw -LiteralPath $stderr)-match 'client-runtime ready'))){break}
    Start-Sleep -Milliseconds 100
}
if(-not(($readyManifest-and(Test-Path -LiteralPath $readyManifest))-or((Test-Path -LiteralPath $stderr)-and((Get-Content -Raw -LiteralPath $stderr)-match 'client-runtime ready')))){Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue;throw "Team $TeamId runtime ready timeout. See $stderr"}
[System.IO.File]::WriteAllText($resolvedPidFile, "$($process.Id)`r`n", (New-Object System.Text.UTF8Encoding $false))
Write-Output $process.Id
