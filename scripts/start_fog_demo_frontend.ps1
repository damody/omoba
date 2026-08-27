param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$WorkingDirectory,
    [Parameter(Mandatory = $true)][string]$PidFile,
    [Parameter(Mandatory = $true)][int]$PlayerId,
    [Parameter(Mandatory = $true)][int]$TeamId,
    [Parameter(Mandatory = $true)][string]$PlayerName,
    [Parameter(Mandatory = $true)][int]$WindowX
)

$resolvedExe = (Resolve-Path -LiteralPath $Exe).Path
$resolvedWorkingDirectory = (Resolve-Path -LiteralPath $WorkingDirectory).Path
$resolvedPidFile = [System.IO.Path]::GetFullPath($PidFile)
[System.IO.Directory]::CreateDirectory([System.IO.Path]::GetDirectoryName($resolvedPidFile)) | Out-Null

$start = [System.Diagnostics.ProcessStartInfo]::new()
$start.FileName = $resolvedExe
$start.WorkingDirectory = $resolvedWorkingDirectory
$start.UseShellExecute = $false
$start.Environment['OMB_PLAYER_ID'] = [string]$PlayerId
$start.Environment['OMB_PLAYER_NAME'] = $PlayerName
$start.Environment['OMB_LOCKSTEP_PLAYER_NAME'] = "fog_demo_player_$PlayerId"
$start.Environment['OMB_TEAM_ID'] = [string]$TeamId
$start.Environment['OMB_STORY'] = 'FOG_2TEAM_DEMO'
$start.Environment['OMFX_LEGACY_AUTOSTART'] = '1'
$start.Environment['OMFX_EXTERNAL_BACKEND'] = '1'
$start.Environment['OMFX_WINDOW_TITLE_SUFFIX'] = "P$PlayerId / Team $TeamId / FOG"
$start.Environment['OMFX_LOG_SUFFIX'] = "fog_p$PlayerId"
$start.Environment['OMFX_WINDOW_X'] = [string]$WindowX
$start.Environment['OMFX_WINDOW_Y'] = '40'
$start.Environment['OMFX_WINDOW_WIDTH'] = '920'
$start.Environment['OMFX_WINDOW_HEIGHT'] = '720'

$process = [System.Diagnostics.Process]::Start($start)
if ($null -eq $process) { throw "無法啟動玩家 $PlayerId 的 omfx" }
[System.IO.File]::WriteAllText($resolvedPidFile, [string]$process.Id)
Write-Output $process.Id
