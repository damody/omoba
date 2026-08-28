param(
 [ValidateSet('headless','visual')][string]$RunMode='headless',
 [ValidateSet('ordered-delay','natural-reorder')][string]$DelayMode='ordered-delay',
 [string]$Team1Profile='uniform-20-100',[string]$Team2Profile='uniform-20-100',
 [ValidateSet('single','isolation','soak')][string]$Preset='single',[UInt64]$Seed=88442211,[int]$DurationSeconds=15,[string]$RunId='',
 [string]$Team1Custom='',[string]$Team2Custom=''
)
$ErrorActionPreference='Stop';if($DurationSeconds-lt1){throw 'DurationSeconds must be positive'};if(-not$RunId){$RunId="netem-$DelayMode-$Team1Profile-$([DateTime]::UtcNow.ToString('yyyyMMddHHmmss'))"}
if($Preset-eq'isolation'){$Team1Profile='high-skew';$Team2Profile='low-skew'}
$env:OMOBA_NETEM='1';$env:OMOBA_NETEM_MODE=$DelayMode;$env:OMOBA_NETEM_SEED=[string]$Seed;$env:OMOBA_NETEM_TEAM1_PROFILE=$Team1Profile;$env:OMOBA_NETEM_TEAM2_PROFILE=$Team2Profile;$env:OMOBA_HEADLESS_SECONDS=[string]$DurationSeconds;$env:OMOBA_RUN_ID=$RunId
$env:OMOBA_NETEM_TEAM1_CUSTOM=$Team1Custom;$env:OMOBA_NETEM_TEAM2_CUSTOM=$Team2Custom
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path;$evidence=Join-Path $root "openspec\changes\simulate-client-rtt-delay\evidence\runs\$RunId"
if(Test-Path -LiteralPath $evidence){throw "RunId already exists and evidence will not be overwritten: $RunId"}
if($Preset-ne'soak'){& cmd.exe /d /c "run_2player.bat $RunMode";exit $LASTEXITCODE}
$env:OMOBA_HEADLESS_SECONDS=[string]$DurationSeconds;$process=Start-Process -FilePath cmd.exe -WorkingDirectory $root -ArgumentList @('/d','/c',"run_2player.bat $RunMode") -WindowStyle Hidden -PassThru
$profiles=@('low-skew','fixed-60','high-skew','bimodal-20-100','low-skew');$segment=[Math]::Max(1,[Math]::Floor($DurationSeconds/$profiles.Count))
for($i=1;$i-lt$profiles.Count;$i++){Start-Sleep -Seconds $segment;if($process.HasExited){throw "launcher exited early with code $($process.ExitCode)"};$tick=0;$timeline=Join-Path $evidence 'server\canonical-timeline.jsonl';if(Test-Path $timeline){$last=Get-Content $timeline -Tail 1|ConvertFrom-Json;$tick=[UInt64]$last.tick};foreach($team in 1,2){& "$PSScriptRoot\send_netem_control.ps1" -Action profile -TeamId $team -Profile $profiles[$i] -AuthoritativeTick $tick}}
$process.WaitForExit();exit $process.ExitCode
