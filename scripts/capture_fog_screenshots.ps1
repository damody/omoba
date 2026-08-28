param([Parameter(Mandatory=$true)][string]$EvidenceDir,[Parameter(Mandatory=$true)][int]$Team1RendererPid,[Parameter(Mandatory=$true)][int]$Team2RendererPid)
$ErrorActionPreference='Stop';Add-Type -AssemblyName System.Drawing
if(-not('FogWindowCapture' -as [type])){Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class FogWindowCapture {
 [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left,Top,Right,Bottom; }
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd,out Rect rect);
 [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd,IntPtr hdc,uint flags);
}
'@}
$markers=@((Join-Path $EvidenceDir 'team-1-runtime\screenshot.tick'),(Join-Path $EvidenceDir 'team-2-runtime\screenshot.tick'))
$deadline=[DateTime]::UtcNow.AddSeconds(30)
while([DateTime]::UtcNow-lt$deadline){if(-not($markers|Where-Object{-not(Test-Path $_)})){break};Start-Sleep -Milliseconds 100}
if($markers|Where-Object{-not(Test-Path $_)}){throw 'authoritative screenshot trigger timeout'}
Start-Sleep -Seconds 5
foreach($item in @(@(1,$Team1RendererPid),@(2,$Team2RendererPid))){
 $team=$item[0];$p=Get-Process -Id $item[1] -ErrorAction Stop
 if($p.MainWindowHandle-eq[IntPtr]::Zero){throw "Team $team renderer has no main window"}
 $rect=New-Object FogWindowCapture+Rect;if(-not[FogWindowCapture]::GetWindowRect($p.MainWindowHandle,[ref]$rect)){throw "GetWindowRect failed for Team $team"}
 $width=$rect.Right-$rect.Left;$height=$rect.Bottom-$rect.Top;if($width-le0-or$height-le0){throw "Invalid renderer window bounds"}
 $bitmap=New-Object Drawing.Bitmap $width,$height;$graphics=[Drawing.Graphics]::FromImage($bitmap);$hdc=$graphics.GetHdc()
 try { if(-not[FogWindowCapture]::PrintWindow($p.MainWindowHandle,$hdc,2)){throw "PrintWindow failed for Team $team"} } finally { $graphics.ReleaseHdc($hdc) }
 $path=Join-Path $EvidenceDir "team-$team-screenshot.png";$bitmap.Save($path,[Drawing.Imaging.ImageFormat]::Png);$graphics.Dispose();$bitmap.Dispose()
}
