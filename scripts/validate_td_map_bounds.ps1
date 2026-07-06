param(
    [string]$Root = "scripts/lua_data"
)

$ErrorActionPreference = "Stop"

$minX = -1400.0
$maxX = 1400.0
$minY = -800.0
$maxY = 800.0

$numberPattern = "-?\d+(?:\.\d+)?"
$coordPattern = "(?:X\s*=\s*(?<x>$numberPattern)\s*,\s*Y\s*=\s*(?<y>$numberPattern)|Y\s*=\s*(?<y>$numberPattern)\s*,\s*X\s*=\s*(?<x>$numberPattern))"
$namePattern = 'Name\s*=\s*"(?<name>[^"]+)"'

$mapFiles = @(Get-ChildItem -Path $Root -Recurse -Filter map.lua |
    Where-Object { $_.Directory.Name -like "TD_*" } |
    Sort-Object FullName)

$failures = @()

if ($mapFiles.Count -eq 0) {
    Write-Host "TD map bounds failed: no TD map.lua files found under '$Root'"
    exit 1
}

foreach ($file in $mapFiles) {
    $text = Get-Content -Raw -LiteralPath $file.FullName
    $lineStarts = New-Object System.Collections.Generic.List[int]
    $lineStarts.Add(0)

    for ($i = 0; $i -lt $text.Length; $i += 1) {
        if ($text[$i] -eq "`n") {
            $lineStarts.Add($i + 1)
        }
    }

    $index = 0
    foreach ($match in [regex]::Matches($text, $coordPattern)) {
        $index += 1
        $x = [double]$match.Groups["x"].Value
        $y = [double]$match.Groups["y"].Value

        if ($x -lt $minX -or $x -gt $maxX -or $y -lt $minY -or $y -gt $maxY) {
            $line = 1
            for ($lineIndex = 0; $lineIndex -lt $lineStarts.Count; $lineIndex += 1) {
                if ($lineStarts[$lineIndex] -gt $match.Index) {
                    break
                }
                $line = $lineIndex + 1
            }

            $nearbyStart = [Math]::Max(0, $match.Index - 500)
            $nearbyLength = $match.Index - $nearbyStart
            $nearbyText = $text.Substring($nearbyStart, $nearbyLength)
            $nameMatches = [regex]::Matches($nearbyText, $namePattern)
            $label = "coordinate #$index"
            if ($nameMatches.Count -gt 0) {
                $label = "$label near '$($nameMatches[$nameMatches.Count - 1].Groups["name"].Value)'"
            }

            $failures += "$($file.FullName):$line $label has ($x,$y), allowed x=$minX..$maxX y=$minY..$maxY"
        }
    }
}

if ($failures.Count -gt 0) {
    Write-Host "TD map bounds failed: $($failures.Count) out-of-range coordinate(s)"
    $failures | ForEach-Object { Write-Host $_ }
    exit 1
}

Write-Host "TD map bounds OK: $($mapFiles.Count) map.lua files checked within x=$minX..$maxX y=$minY..$maxY"
