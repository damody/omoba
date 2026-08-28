param([Parameter(Mandatory=$true)][int]$ProcessId,[Parameter(Mandatory=$true)][string]$ExpectedExe,[string]$ControlAddr='127.0.0.1:63200')
$ErrorActionPreference='Stop';$process=Get-Process -Id $ProcessId -ErrorAction SilentlyContinue;if(-not$process){exit 0};$expected=(Resolve-Path -LiteralPath $ExpectedExe).Path;if($process.Path-ne$expected){throw 'netem proxy PID/path mismatch'}
& "$PSScriptRoot\send_netem_control.ps1" -ControlAddr $ControlAddr -Action shutdown;$null=$process.WaitForExit(7000);if(-not$process.HasExited){Stop-Process -Id $ProcessId -Force}
