param(
 [Parameter(Mandatory=$true)][int]$ServerPid,[Parameter(Mandatory=$true)][int]$Team1RuntimePid,[Parameter(Mandatory=$true)][int]$Team2RuntimePid,
 [Parameter(Mandatory=$true)][int]$Team1RendererPid,[Parameter(Mandatory=$true)][string]$RuntimeExe,[Parameter(Mandatory=$true)][string]$RendererExe,
 [Parameter(Mandatory=$true)][string]$EvidenceDir
)
$ErrorActionPreference='Stop';$events=@()
function AssertPath($processId,$exe){$p=Get-Process -Id $processId -ErrorAction Stop;if($p.Path-ne(Resolve-Path $exe).Path){throw 'PID/path mismatch'};$p}
$renderer=AssertPath $Team1RendererPid $RendererExe;$null=$renderer.CloseMainWindow();if(-not$renderer.WaitForExit(5000)){Stop-Process -Id $Team1RendererPid -Force}
Start-Sleep -Milliseconds 500;$runtime=AssertPath $Team1RuntimePid $RuntimeExe;$events+=[ordered]@{event='team-1-renderer-stopped';runtime_alive=(-not$runtime.HasExited)}
$pidFile=Join-Path $EvidenceDir 'team-1-renderer-restart.pid';$newPid=& "$PSScriptRoot\start_fog_demo_frontend.ps1" -Exe $RendererExe -WorkingDirectory 'omfx' -PidFile $pidFile -PlayerId 1 -TeamId 1 -PlayerName player1 -WindowX 20 -PresentationAddr '127.0.0.1:62001'
Start-Sleep -Seconds 2;$null=AssertPath ([int]$newPid) $RendererExe;$events+=[ordered]@{event='team-1-renderer-reconnected';renderer_pid=[int]$newPid;runtime_pid=$Team1RuntimePid}
$runtime=AssertPath $Team1RuntimePid $RuntimeExe;Stop-Process -Id $runtime.Id -Force;Start-Sleep -Milliseconds 500
$events+=[ordered]@{event='team-1-runtime-stopped';server_alive=[bool](Get-Process -Id $ServerPid -ErrorAction SilentlyContinue);team_2_runtime_alive=[bool](Get-Process -Id $Team2RuntimePid -ErrorAction SilentlyContinue)}
$restarted=Get-Process -Id ([int]$newPid) -ErrorAction SilentlyContinue;if($restarted-and$restarted.Path-eq(Resolve-Path $RendererExe).Path){$null=$restarted.CloseMainWindow();if(-not$restarted.WaitForExit(3000)){Stop-Process -Id $restarted.Id -Force}}
$events|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $EvidenceDir 'lifecycle.json') -Encoding utf8
