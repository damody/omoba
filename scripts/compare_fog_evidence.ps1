param([Parameter(Mandatory=$true)][string]$EvidenceDir)
$ErrorActionPreference='Stop'
if(-not('FogByteScanner' -as [type])){Add-Type -TypeDefinition @'
using System.IO;
public static class FogByteScanner {
 public static bool Contains(string path, byte[] needle) {
  byte[] buffer=new byte[1024*1024+needle.Length]; int carry=0;
  using(var stream=new FileStream(path,FileMode.Open,FileAccess.Read,FileShare.ReadWrite)) {
   int read; while((read=stream.Read(buffer,carry,1024*1024))>0) {
    int total=carry+read;
    int limit=total-needle.Length; int i=0;
    while(i<=limit) {
     i=System.Array.IndexOf(buffer,needle[0],i,limit-i+1);
     if(i<0)break;
     int j=1;for(;j<needle.Length&&buffer[i+j]==needle[j];j++);
     if(j==needle.Length)return true;
     i++;
    }
    carry=System.Math.Min(needle.Length-1,total); System.Buffer.BlockCopy(buffer,total-carry,buffer,0,carry);
   }
  }
  return false;
 }
}
'@}
$root=[IO.Path]::GetFullPath($EvidenceDir)
$secretPath=Join-Path $root 'server\sentinels.secret.json'
$required=@('manifest.json','server\canonical-timeline.jsonl','server\disclosure-matrix.jsonl','server\three-way-checkpoints.json','team-1-runtime\manifest.json','team-2-runtime\manifest.json','team-1-runtime\team-frame.capture','team-2-runtime\team-frame.capture','team-1-runtime\filtered-world.latest.json','team-2-runtime\filtered-world.latest.json','team-1-runtime\presentation.capture','team-2-runtime\presentation.capture')
$gates=@(); $missing=@($required|Where-Object{-not(Test-Path -LiteralPath (Join-Path $root $_))})
$gates += [ordered]@{name='required-artifacts';status=$(if($missing.Count){'UNVERIFIED'}else{'PASS'});missing=$missing}
if (-not(Test-Path -LiteralPath $secretPath)) { $gates += [ordered]@{name='sentinel-secrets';status='UNVERIFIED';reason='missing server-only sentinel file'} }
else {
  $secrets=Get-Content -Raw -LiteralPath $secretPath|ConvertFrom-Json
  foreach($team in 1,2){
    $opponent=if($team-eq 1){$secrets.team_2_hex}else{$secrets.team_1_hex}
    $hex=[string]$opponent; $needle=New-Object byte[] ($hex.Length/2); for($n=0;$n-lt$needle.Length;$n++){$needle[$n]=[Convert]::ToByte($hex.Substring($n*2,2),16)}
    $hits=@()
    $scanFiles=@(Get-ChildItem -LiteralPath (Join-Path $root "team-$team-runtime") -File -ErrorAction SilentlyContinue)
    $scanFiles+=@(Get-ChildItem -LiteralPath $root -File -Filter "team-$team-*.dmp" -ErrorAction SilentlyContinue)
    $scanFiles|ForEach-Object{
      if([FogByteScanner]::Contains($_.FullName,$needle)){$hits+=$_.FullName}
    }
    $gates += [ordered]@{name="team-$team-opponent-sentinel-absence";status=$(if($hits.Count){'FAIL'}elseif($scanFiles.Count){'PASS'}else{'UNVERIFIED'});hits=$hits;scanned_files=@($scanFiles.FullName);false_positive_exclusion='server-only secret directory excluded; exact 128-bit byte sequence only'}
    $dumpMeta=Get-ChildItem -LiteralPath $root -Filter "team-$team-*.dmp.json" -ErrorAction SilentlyContinue
    $dumpRows=@($dumpMeta|ForEach-Object{Get-Content -Raw -LiteralPath $_.FullName|ConvertFrom-Json})
    $gates += [ordered]@{name="team-$team-memory-capture";status=$(if($dumpRows -and ($dumpRows.status -contains 'CAPTURED')){'PASS'}else{'UNVERIFIED'});pid=$(if($dumpRows){$dumpRows.pid}else{$null})}
  }
}
$checkpointRows=@()
foreach($team in 1,2){
  $expectedPath=Join-Path $root "server\team-$team\expected-timeline.jsonl"; $clientPath=Join-Path $root "team-$team-runtime\filtered-timeline.jsonl"
  $expected=@{}; if(Test-Path $expectedPath){Get-Content $expectedPath|ForEach-Object{try{$v=$_|ConvertFrom-Json}catch{return};if($v.expected_pre_repair_hash){$expected["$($v.replica_tick):$($v.team_sequence)"]=$v.expected_pre_repair_hash}}}
  $covered=0;$mismatch=0;if(Test-Path $clientPath){Get-Content $clientPath|ForEach-Object{try{$v=$_|ConvertFrom-Json}catch{return};$key="$($v.replica_tick):$($v.team_sequence)";if($expected.ContainsKey($key)){$covered++;if($expected[$key]-ne$v.post_repair_hash){$mismatch++};$checkpointRows+=[ordered]@{team_id=$team;key=$key;server_expected=$expected[$key];external_runtime=$v.post_repair_hash;parity=($expected[$key]-eq$v.post_repair_hash)}}}}
  $runtimeEventsPath=Join-Path $root "team-$team-runtime\network-events.jsonl";$runtimeEvents=@();if(Test-Path $runtimeEventsPath){$runtimeEvents=@(Get-Content $runtimeEventsPath|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})}
  $rebasesForCheckpoint=@($runtimeEvents|Where-Object{$_.kind-eq'rebase-applied'});$lastRebaseTick=($rebasesForCheckpoint|Measure-Object -Property replica_tick -Maximum).Maximum
  $postRecoveryParity=@($checkpointRows|Where-Object{$_.team_id-eq$team-and$_.parity-and$lastRebaseTick-and[UInt64]($_.key-split':')[0]-gt[UInt64]$lastRebaseTick}).Count-gt0
  $postRecoveryApplied=@($runtimeEvents|Where-Object{$_.kind-eq'frame-applied'-and$lastRebaseTick-and[UInt64]$_.replica_tick-gt[UInt64]$lastRebaseTick}).Count-gt0
  $recoveryConfirmed=$postRecoveryParity-or$postRecoveryApplied
  $checkpointPass=$mismatch-eq0-or($rebasesForCheckpoint.Count-gt0-and$recoveryConfirmed)
  $gates += [ordered]@{name="team-$team-checkpoint-coverage";status=$(if($covered-eq0){'UNVERIFIED'}elseif($checkpointPass){'PASS'}else{'FAIL'});covered=$covered;mismatch=$mismatch;recovered_after_verified_rebase=$recoveryConfirmed;post_rebase_checkpoint=$postRecoveryParity;post_rebase_frame_applied=$postRecoveryApplied}
}
$checkpointRows|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $root 'checkpoint-comparison.json') -Encoding utf8
$manifestPath=Join-Path $root 'manifest.json';$gates += [ordered]@{name='process-lifecycle';status=$(if(Test-Path $manifestPath){'PASS'}else{'UNVERIFIED'});manifest=$manifestPath}
$manifest=if(Test-Path $manifestPath){Get-Content -Raw $manifestPath|ConvertFrom-Json}else{$null}
if($manifest-and$manifest.netem-and$manifest.netem.mode-ne'direct'){
 $proxyPath=Join-Path $root 'proxy-evidence.json'
 if(-not(Test-Path $proxyPath)){$gates += [ordered]@{name='netem-evidence';status='UNVERIFIED';missing=$proxyPath}}
 else{
  $proxy=Get-Content -Raw $proxyPath|ConvertFrom-Json
  $gates += [ordered]@{name='netem-process-status';status=$(if($proxy.status-eq'PASS'){'PASS'}else{'FAIL'});failure=$proxy.failure}
  $routes=@($proxy.routes);$gates += [ordered]@{name='netem-route-isolation';status=$(if($routes.Count-eq2-and$manifest.netem.team_1_route-ne$manifest.netem.team_2_route){'PASS'}else{'FAIL'});route_count=$routes.Count}
  $directions=@($routes|ForEach-Object{$_.client_to_server;$_.server_to_client})
  $rttPass=$directions.Count-eq4-and@($directions|Where-Object{$_.released_datagrams-eq0-or$_.scheduled_rtt_min_ms-lt20-or$_.scheduled_rtt_max_ms-gt100}).Count-eq0
  $gates += [ordered]@{name='netem-rtt-range';status=$(if($rttPass){'PASS'}else{'FAIL'});directions=$directions.Count}
  $histPass=$true;foreach($route in $routes){foreach($direction in @($route.client_to_server,$route.server_to_client)){for($i=0;$i-lt20;$i++){if([UInt64]$route.weights[$i]-gt0-and[UInt64]$direction.observed_histogram[$i]-eq0){$histPass=$false}}}}
  $gates += [ordered]@{name='netem-histogram-coverage';status=$(if($histPass){'PASS'}else{'UNVERIFIED'})}
  $reorders=($directions|Measure-Object -Property reordered_datagrams -Sum).Sum
  $reorderPass=if($manifest.netem.mode-eq'ordered-delay'){$reorders-eq0}else{$reorders-gt0}
  $gates += [ordered]@{name='netem-reorder-mode';status=$(if($reorderPass){'PASS'}elseif($manifest.netem.mode-eq'natural-reorder'){'UNVERIFIED'}else{'FAIL'});mode=$manifest.netem.mode;reordered_datagrams=$reorders}
  $budgetPass=@($directions|Where-Object{$_.packets_high_watermark-gt4096-or$_.bytes_high_watermark-gt33554432}).Count-eq0
  $gates += [ordered]@{name='netem-queue-budget';status=$(if($budgetPass){'PASS'}else{'FAIL'})}
  foreach($team in 1,2){
   $eventsPath=Join-Path $root "team-$team-runtime\network-events.jsonl";$events=@();if(Test-Path $eventsPath){$events=@(Get-Content $eventsPath|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})}
   $applied=@($events|Where-Object{$_.kind-eq'frame-applied'}|Sort-Object team_sequence);$unique=@($applied.team_sequence|Sort-Object -Unique)
   $sequencePass=$applied.Count-gt0-and$unique.Count-eq$applied.Count;for($i=1;$i-lt$unique.Count;$i++){if([UInt64]$unique[$i]-ne([UInt64]$unique[$i-1]+1)){$sequencePass=$false}}
   $unsafe=@($events|Where-Object{$_.kind-in@('wrong-team-rejected','frame-rejected')})
   $gates += [ordered]@{name="team-$team-netem-sequence";status=$(if(-not(Test-Path $eventsPath)){'UNVERIFIED'}elseif($sequencePass-and$unsafe.Count-eq0){'PASS'}else{'FAIL'});applied=$applied.Count;unsafe=$unsafe.Count}
   $move=@($events|Where-Object{$_.kind-eq'input-forwarded'-and$_.code-eq'FORWARDED'})
   $moveApplied=Test-Path -LiteralPath (Join-Path $root "team-$team-runtime\scripted-move-applied.tick")
   $gates += [ordered]@{name="team-$team-delayed-move-input";status=$(if($move.Count-gt0-and$moveApplied){'PASS'}else{'UNVERIFIED'});forwarded=$move.Count;applied=$moveApplied}
   $hiddenSubmitted=Test-Path -LiteralPath (Join-Path $root "team-$team-runtime\scripted-hidden-target-submitted.tick")
   $hiddenRejected=@($events|Where-Object{$_.kind-eq'secure-input-result'-and$_.code-eq'SERVER_INVALID_TARGET'})
   $gates += [ordered]@{name="team-$team-hidden-target-rejection";status=$(if($hiddenSubmitted-and$hiddenRejected.Count-gt0){'PASS'}else{'UNVERIFIED'});submitted=$hiddenSubmitted;rejected=$hiddenRejected.Count}
   $rebases=@($events|Where-Object{$_.kind-eq'rebase-applied'});$safeRebases=0;foreach($rebase in $rebases){if(@($applied|Where-Object{$_.team_sequence-ge$rebase.team_sequence}).Count-gt0){$safeRebases++}}
   $gates += [ordered]@{name="team-$team-unintended-rebase";status=$(if($safeRebases-eq$rebases.Count){'PASS'}else{'FAIL'});count=($rebases.Count-$safeRebases);verified_recovery_count=$safeRebases}
  }
 }
}
$disclosurePath=Join-Path $root 'server\disclosure-matrix.jsonl'
foreach($team in 1,2){
 $rows=@();if(Test-Path $disclosurePath){$rows=@(Get-Content $disclosurePath|ForEach-Object{try{$_|ConvertFrom-Json}catch{}}|Where-Object{$_.team_id-eq$team}|Sort-Object team_sequence)}
 $monotonic=$rows.Count-gt0;for($i=1;$i-lt$rows.Count;$i++){if([UInt64]$rows[$i].team_sequence-ne([UInt64]$rows[$i-1].team_sequence+1)-or[UInt64]$rows[$i].replica_tick-lt[UInt64]$rows[$i-1].replica_tick){$monotonic=$false}}
 $kinds=@($rows|ForEach-Object{$_.transitions}|ForEach-Object{$_.kind}|Sort-Object -Unique)
 $gates += [ordered]@{name="team-$team-disclosure-timeline";status=$(if($monotonic-and($kinds-contains'Reveal')){'PASS'}else{'UNVERIFIED'});monotonic=$monotonic;kinds=$kinds;coverage_note='Hide/Forget/LastKnown may be absent in a short profile smoke; dedicated movement and visual scenarios require them.'}
}
$screens=@(Get-ChildItem -LiteralPath $root -Recurse -Filter 'screenshot.tick' -ErrorAction SilentlyContinue);$images=@(Get-ChildItem -LiteralPath $root -Filter 'team-*-screenshot.png' -ErrorAction SilentlyContinue);@($screens+$images)|Select-Object FullName|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $root 'screenshot-index.json') -Encoding utf8
$mode=if($manifest){$manifest.mode}else{'unknown'};$screenPass=$screens.Count-ge 2-and($mode-ne'visual'-or$images.Count-ge 2)
$gates += [ordered]@{name='screenshot-triggers';status=$(if($screenPass){'PASS'}else{'UNVERIFIED'});trigger_count=$screens.Count;image_count=$images.Count}
if($mode-eq'visual'){
 $rendererMeta=@(Get-ChildItem -LiteralPath $root -Filter 'team-*-renderer.dmp.json' -ErrorAction SilentlyContinue|ForEach-Object{Get-Content -Raw -LiteralPath $_.FullName|ConvertFrom-Json})
 $gates += [ordered]@{name='renderer-memory-captures';status=$(if($rendererMeta.Count-eq2-and@($rendererMeta|Where-Object{$_.status-ne'CAPTURED'}).Count-eq0){'PASS'}else{'UNVERIFIED'});captured=$rendererMeta.Count}
 $imageHashes=@($images|ForEach-Object{$hash=[Security.Cryptography.SHA256]::Create();$stream=[IO.File]::OpenRead($_.FullName);try{-join($hash.ComputeHash($stream)|ForEach-Object{$_.ToString('x2')})}finally{$stream.Dispose();$hash.Dispose()}})
 $gates += [ordered]@{name='asymmetric-team-images';status=$(if($imageHashes.Count-eq2-and$imageHashes[0]-ne$imageHashes[1]){'PASS'}else{'FAIL'});hashes=$imageHashes}
 $life=Join-Path $root 'lifecycle.json';if(Test-Path $life){$events=Get-Content -Raw $life|ConvertFrom-Json;$lifePass=$events.Count-ge3-and$events[0].runtime_alive-and$events[2].server_alive-and$events[2].team_2_runtime_alive;$gates += [ordered]@{name='renderer-restart-runtime-isolation';status=$(if($lifePass){'PASS'}else{'FAIL'});events=$events}}else{$gates += [ordered]@{name='renderer-restart-runtime-isolation';status='UNVERIFIED';reason='missing lifecycle.json'}}
}
$status=if($gates.status-contains'FAIL'){'FAIL'}elseif($gates.status-contains'UNVERIFIED'){'UNVERIFIED'}else{'PASS'}
$verdict=[ordered]@{schema_version=1;verdict=$status;generated_utc=[DateTime]::UtcNow.ToString('o');blocking_gates=$gates}
$verdict|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $root 'verdict.json') -Encoding utf8
$verdict|ConvertTo-Json -Depth 8
if($status-eq'PASS'){exit 0}elseif($status-eq'FAIL'){exit 1}else{exit 2}
