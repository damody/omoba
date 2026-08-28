param(
 [Parameter(Mandatory=$true)][string]$EvidenceDir,
 [Parameter(Mandatory=$true)][int]$ServerPid,
 [Parameter(Mandatory=$true)][int]$Team1RuntimePid,
 [Parameter(Mandatory=$true)][int]$Team2RuntimePid,
 [int]$Team1RendererPid=0,[int]$Team2RendererPid=0,
 [Parameter(Mandatory=$true)][string]$ServerExe,
 [Parameter(Mandatory=$true)][string]$RuntimeExe,
 [string]$RendererExe='',[string]$Mode='headless',
 [int]$ProxyPid=0,[string]$ProxyExe='',[string]$NetemMode='direct',[UInt64]$NetemSeed=0,
 [string]$Team1Route='',[string]$Team2Route='',[string]$Team1Profile='',[string]$Team2Profile=''
)
$ErrorActionPreference='Stop'; [IO.Directory]::CreateDirectory($EvidenceDir)|Out-Null
function HashFile($path){$sha=[Security.Cryptography.SHA256]::Create();$stream=[IO.File]::OpenRead($path);try{return -join($sha.ComputeHash($stream)|ForEach-Object{$_.ToString('x2')})}finally{$stream.Dispose();$sha.Dispose()}}
function Proc($processId,$expected,$role,$team,$player){
 if($processId-le 0){return $null}; $p=Get-Process -Id $processId -ErrorAction Stop; $path=(Resolve-Path -LiteralPath $expected).Path
 if($p.Path-ne$path){throw "$role PID/path mismatch"}
 [ordered]@{role=$role;pid=$processId;path=$path;sha256=(HashFile $path);team_id=$team;player_id=$player}
}
$manifest=[ordered]@{schema_version=1;mode=$Mode;created_utc=[DateTime]::UtcNow.ToString('o');rustc=(& rustc --version);netem=[ordered]@{mode=$NetemMode;seed=$NetemSeed;team_1_route=$Team1Route;team_2_route=$Team2Route;team_1_profile=$Team1Profile;team_2_profile=$Team2Profile};ports=@{server='127.0.0.1:50061';team_1_presentation='127.0.0.1:62001';team_2_presentation='127.0.0.1:62002'};processes=@(
 (Proc $ServerPid $ServerExe 'authoritative-server' 0 0),
 (Proc $ProxyPid $ProxyExe 'netem-proxy' 0 0),
 (Proc $Team1RuntimePid $RuntimeExe 'team-runtime' 1 1),
 (Proc $Team2RuntimePid $RuntimeExe 'team-runtime' 2 2),
 (Proc $Team1RendererPid $RendererExe 'renderer' 1 1),
 (Proc $Team2RendererPid $RendererExe 'renderer' 2 2)
)|Where-Object{$_}}
$manifest|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $EvidenceDir 'manifest.json') -Encoding utf8
