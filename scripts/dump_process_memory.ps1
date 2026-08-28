param(
    [Parameter(Mandatory=$true)][int]$ProcessId,
    [Parameter(Mandatory=$true)][string]$ExpectedExe,
    [Parameter(Mandatory=$true)][string]$OutputPath,
    [Parameter(Mandatory=$true)][ValidateSet('runtime','renderer')][string]$Role
)
$ErrorActionPreference='Stop'
$result=[ordered]@{ schema_version=1; pid=$ProcessId; role=$Role; status='UNVERIFIED'; method='DbgHelp MiniDumpWriteDump full-memory'; tool_version=[Environment]::OSVersion.VersionString; binary_path=''; binary_sha256=''; dump_path=[IO.Path]::GetFullPath($OutputPath); reason='' }
try {
    $expected=(Resolve-Path -LiteralPath $ExpectedExe).Path
    $process=Get-Process -Id $ProcessId -ErrorAction Stop
    if ($process.Path -ne $expected) { throw "PID executable mismatch" }
    $result.binary_path=$expected
    $sha=[Security.Cryptography.SHA256]::Create();$stream=[IO.File]::OpenRead($expected)
    try { $result.binary_sha256=-join($sha.ComputeHash($stream)|ForEach-Object{$_.ToString('x2')}) } finally { $stream.Dispose();$sha.Dispose() }
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($result.dump_path))|Out-Null
    if(-not('FogMiniDump' -as [type])){Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Runtime.InteropServices;
public static class FogMiniDump {
 [DllImport("Dbghelp.dll")] static extern bool MiniDumpWriteDump(IntPtr hProcess,uint processId,SafeHandle hFile,uint dumpType,IntPtr ex,IntPtr user,IntPtr callback);
 public static bool Write(System.Diagnostics.Process process, FileStream stream) { return MiniDumpWriteDump(process.Handle,(uint)process.Id,stream.SafeFileHandle,2,IntPtr.Zero,IntPtr.Zero,IntPtr.Zero); }
}
'@}
    $dumpStream=[IO.File]::Open($result.dump_path,[IO.FileMode]::Create,[IO.FileAccess]::ReadWrite,[IO.FileShare]::Read)
    try { if(-not[FogMiniDump]::Write($process,$dumpStream)){throw "MiniDumpWriteDump failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"} } finally { $dumpStream.Dispose() }
    $result.status='CAPTURED'
} catch { $result.reason=$_.Exception.Message }
$result|ConvertTo-Json -Depth 4|Set-Content -LiteralPath ($result.dump_path+'.json') -Encoding utf8
if ($result.status -ne 'CAPTURED') { exit 2 }
