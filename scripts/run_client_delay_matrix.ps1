param([ValidateSet('smoke','ordered','reorder','all')][string]$Matrix='smoke',[UInt64]$Seed=88442211)
$ErrorActionPreference='Stop';$profiles=@('fixed-20','fixed-60','fixed-100','uniform-20-100','low-skew','high-skew','bimodal-20-100');$matrixId=[DateTime]::UtcNow.ToString('yyyyMMddHHmmssfff')
function Run($mode,$profile,$seconds,$suffix,$custom=''){& "$PSScriptRoot\run_client_delay_scenario.ps1" -RunMode headless -DelayMode $mode -Team1Profile $profile -Team2Profile $profile -Team1Custom $custom -Team2Custom $custom -DurationSeconds $seconds -Seed $Seed -RunId "netem-$matrixId-$suffix-$profile";if($LASTEXITCODE){throw "$suffix/$profile failed: $LASTEXITCODE"}}
if($Matrix-in@('smoke','all')){foreach($profile in $profiles){Run ordered-delay $profile 15 smoke};$custom=(Resolve-Path "$PSScriptRoot\..\openspec\changes\simulate-client-rtt-delay\fixtures\custom-valid.json").Path;Run ordered-delay custom-20-bin 15 smoke $custom}
if($Matrix-in@('ordered','all')){foreach($profile in $profiles){Run ordered-delay $profile 300 ordered}}
if($Matrix-in@('reorder','all')){foreach($profile in $profiles){Run natural-reorder $profile 300 reorder}}
