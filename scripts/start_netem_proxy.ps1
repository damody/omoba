param(
 [Parameter(Mandatory=$true)][string]$Exe,[Parameter(Mandatory=$true)][string]$PidFile,[Parameter(Mandatory=$true)][string]$EvidenceDir,
 [string]$ServerAddr='127.0.0.1:50061',[string]$Team1ClientBind='127.0.0.1:63001',[string]$Team2ClientBind='127.0.0.1:63002',
 [string]$Team1UpstreamBind='127.0.0.1:63101',[string]$Team2UpstreamBind='127.0.0.1:63102',[string]$ControlBind='127.0.0.1:63200',
 [ValidateSet('ordered-delay','natural-reorder')][string]$DelayMode='ordered-delay',[string]$Team1Profile='uniform-20-100',[string]$Team2Profile='uniform-20-100',[string]$Team1Custom='',[string]$Team2Custom='',[UInt64]$Seed=88442211
)
$ErrorActionPreference='Stop';$exePath=(Resolve-Path -LiteralPath $Exe).Path;$pidPath=[IO.Path]::GetFullPath($PidFile);$root=[IO.Path]::GetFullPath($EvidenceDir);[IO.Directory]::CreateDirectory($root)|Out-Null;[IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($pidPath))|Out-Null
$stdout=Join-Path $root 'proxy.stdout.log';$stderr=Join-Path $root 'proxy.stderr.log';$args=@('--server',$ServerAddr,'--control-bind',$ControlBind,'--team1-client-bind',$Team1ClientBind,'--team2-client-bind',$Team2ClientBind,'--team1-upstream-bind',$Team1UpstreamBind,'--team2-upstream-bind',$Team2UpstreamBind,'--team1-profile',$Team1Profile,'--team2-profile',$Team2Profile,'--mode',$DelayMode,'--seed',[string]$Seed,'--evidence-dir',$root)
if($Team1Custom){$args+=@('--team1-custom',[IO.Path]::GetFullPath($Team1Custom))};if($Team2Custom){$args+=@('--team2-custom',[IO.Path]::GetFullPath($Team2Custom))}
$process=Start-Process -FilePath $exePath -WorkingDirectory (Split-Path -Parent $exePath) -ArgumentList $args -RedirectStandardOutput $stdout -RedirectStandardError $stderr -WindowStyle Hidden -PassThru
$deadline=[DateTime]::UtcNow.AddSeconds(30);while([DateTime]::UtcNow-lt$deadline){if($process.HasExited){throw "netem proxy exited during startup: $(Get-Content -Raw $stderr -ErrorAction SilentlyContinue)"};if((Get-Content -Raw $stdout -ErrorAction SilentlyContinue)-match'netem-proxy ready'){break};Start-Sleep -Milliseconds 100}
if((Get-Content -Raw $stdout -ErrorAction SilentlyContinue)-notmatch'netem-proxy ready'){Stop-Process -Id $process.Id -Force;throw 'netem proxy ready timeout'}
[IO.File]::WriteAllText($pidPath,"$($process.Id)`r`n",(New-Object Text.UTF8Encoding($false)));$process.Id
