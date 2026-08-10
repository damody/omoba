param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('check', 'stage-dll', 'stage-backend-spawn')]
    [string]$Action,

    [ValidateSet('script-dll', 'backend', 'frontend')]
    [string]$Artifact = '',

    [ValidateSet('debug', 'release')]
    [string]$Profile = 'debug'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
$RepoRoot = Split-Path -Parent $ScriptDir

function Join-RepoPath {
    param([Parameter(Mandatory = $true)][string]$RelativePath)
    return [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $RelativePath))
}

function New-InputSet {
    param(
        [string[]]$Files = @(),
        [string[]]$Directories = @()
    )

    return @{
        Files = $Files
        Directories = $Directories
    }
}

function Merge-InputSets {
    param([Parameter(ValueFromRemainingArguments = $true)]$Sets)

    $files = @()
    $directories = @()
    foreach ($set in $Sets) {
        if ($null -eq $set) { continue }
        $files += @($set.Files)
        $directories += @($set.Directories)
    }

    return New-InputSet -Files ($files | Select-Object -Unique) -Directories ($directories | Select-Object -Unique)
}

function Get-NewestInput {
    param(
        [string[]]$Files,
        [string[]]$Directories
    )

    $newest = $null

    foreach ($relativePath in $Files) {
        $path = Join-RepoPath $relativePath
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "input file missing: $relativePath"
        }

        $item = Get-Item -LiteralPath $path
        if ($null -eq $newest -or $item.LastWriteTimeUtc -gt $newest.Time) {
            $newest = [pscustomobject]@{ Path = $relativePath; Time = $item.LastWriteTimeUtc }
        }
    }

    foreach ($relativePath in $Directories) {
        $path = Join-RepoPath $relativePath
        if (-not (Test-Path -LiteralPath $path -PathType Container)) {
            throw "input directory missing: $relativePath"
        }

        foreach ($item in Get-ChildItem -LiteralPath $path -Recurse -File -Force) {
            if ($null -eq $newest -or $item.LastWriteTimeUtc -gt $newest.Time) {
                $newest = [pscustomobject]@{ Path = $item.FullName.Substring($RepoRoot.Length + 1); Time = $item.LastWriteTimeUtc }
            }
        }
    }

    if ($null -eq $newest) {
        throw 'no input files found'
    }

    return $newest
}

function Test-SameFileHash {
    param(
        [Parameter(Mandatory = $true)][string]$LeftPath,
        [Parameter(Mandatory = $true)][string]$RightPath
    )

    if (-not (Test-Path -LiteralPath $LeftPath -PathType Leaf)) { return $false }
    if (-not (Test-Path -LiteralPath $RightPath -PathType Leaf)) { return $false }

    $left = Get-FileHash -Algorithm SHA256 -LiteralPath $LeftPath
    $right = Get-FileHash -Algorithm SHA256 -LiteralPath $RightPath
    return $left.Hash -eq $right.Hash
}

function Test-ArtifactFresh {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)]$Config
    )

    $outputPath = Join-RepoPath $Config.Output
    if (-not (Test-Path -LiteralPath $outputPath -PathType Leaf)) {
        Write-Host "stale: $Name output missing: $($Config.Output)"
        return 1
    }

    if ($Name -eq 'backend-debug') {
        $releaseBackendPath = Join-RepoPath 'omb/target/release/omobab.exe'
        if (Test-SameFileHash -LeftPath $outputPath -RightPath $releaseBackendPath) {
            Write-Host 'stale: backend-debug output matches release spawn copy'
            return 1
        }
    }

    $output = Get-Item -LiteralPath $outputPath

    if ($Config.ContainsKey('RequiredFeatures')) {
        $fingerprintDir = Join-RepoPath $Config.FingerprintDir
        if (-not (Test-Path -LiteralPath $fingerprintDir -PathType Container)) {
            Write-Host "stale: $Name fingerprint directory missing: $($Config.FingerprintDir)"
            return 1
        }

        $fingerprints = @(Get-ChildItem -LiteralPath $fingerprintDir -Recurse -File -Filter $Config.FingerprintFile |
            Sort-Object LastWriteTimeUtc -Descending)
        if ($fingerprints.Count -eq 0) {
            Write-Host "stale: $Name fingerprint missing: $($Config.FingerprintFile)"
            return 1
        }

        $latestFingerprint = $fingerprints[0]
        $metadata = Get-Content -Raw -LiteralPath $latestFingerprint.FullName | ConvertFrom-Json
        $features = [string]$metadata.features
        foreach ($feature in $Config.RequiredFeatures) {
            if (-not $features.Contains("`"$feature`"")) {
                Write-Host "stale: $Name latest fingerprint missing feature '$feature': $($latestFingerprint.FullName.Substring($RepoRoot.Length + 1))"
                return 1
            }
        }
    }

    $newestInput = Get-NewestInput -Files $Config.Inputs.Files -Directories $Config.Inputs.Directories

    if ($newestInput.Time -gt $output.LastWriteTimeUtc) {
        Write-Host "stale: $Name input newer than output: $($newestInput.Path)"
        return 1
    }

    Write-Host "fresh: $Name output is up-to-date: $($Config.Output)"
    return 0
}

function Stage-BaseContentDll {
    param([Parameter(Mandatory = $true)][string]$SourceProfile)

    $sourceRelative = "scripts/target/$SourceProfile/base_content.dll"
    $destinationRelative = 'scripts/base_content.dll'
    $sourcePath = Join-RepoPath $sourceRelative
    $destinationPath = Join-RepoPath $destinationRelative

    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
        throw "source DLL missing: $sourceRelative"
    }

    $source = Get-Item -LiteralPath $sourcePath
    if (Test-Path -LiteralPath $destinationPath -PathType Leaf) {
        $destination = Get-Item -LiteralPath $destinationPath
        if ($destination.LastWriteTimeUtc -ge $source.LastWriteTimeUtc -and (Test-SameFileHash -LeftPath $sourcePath -RightPath $destinationPath)) {
            Write-Host "fresh: staged DLL is up-to-date: $destinationRelative"
            return 0
        }
    }

    $destinationDir = Split-Path -Parent $destinationPath
    if (-not (Test-Path -LiteralPath $destinationDir -PathType Container)) {
        New-Item -ItemType Directory -Path $destinationDir | Out-Null
    }

    Copy-Item -LiteralPath $sourcePath -Destination $destinationPath -Force
    (Get-Item -LiteralPath $destinationPath).LastWriteTimeUtc = $source.LastWriteTimeUtc
    Write-Host "staged: copied base_content.dll ($SourceProfile) to scripts/"
    return 0
}

function Stage-ReleaseBackendForSpawn {
    $sourceRelative = 'omb/target/release/omobab.exe'
    $destinationRelative = 'omb/target/debug/omobab.exe'
    $sourcePath = Join-RepoPath $sourceRelative
    $destinationPath = Join-RepoPath $destinationRelative

    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
        throw "source backend missing: $sourceRelative"
    }

    $source = Get-Item -LiteralPath $sourcePath
    if (Test-Path -LiteralPath $destinationPath -PathType Leaf) {
        $destination = Get-Item -LiteralPath $destinationPath
        if ($destination.LastWriteTimeUtc -ge $source.LastWriteTimeUtc -and (Test-SameFileHash -LeftPath $sourcePath -RightPath $destinationPath)) {
            Write-Host "fresh: release backend spawn copy is up-to-date: $destinationRelative"
            return 0
        }
    }

    $destinationDir = Split-Path -Parent $destinationPath
    if (-not (Test-Path -LiteralPath $destinationDir -PathType Container)) {
        New-Item -ItemType Directory -Path $destinationDir | Out-Null
    }

    Copy-Item -LiteralPath $sourcePath -Destination $destinationPath -Force
    (Get-Item -LiteralPath $destinationPath).LastWriteTimeUtc = $source.LastWriteTimeUtc
    Write-Host 'staged: copied omobab.exe (release) to omb/target/debug/'
    return 0
}

$common = New-InputSet -Files @(
    'rust-toolchain.toml'
)

$scriptAbi = New-InputSet -Files @(
    'scripts/script-abi/Cargo.toml'
) -Directories @(
    'scripts/script-abi/src'
)

$omobaCore = New-InputSet -Files @(
    'omoba-core/Cargo.toml',
    'omoba-core/Cargo.lock',
    'omoba-core/build.rs',
    'proto/game.proto'
) -Directories @(
    'omoba-core/src'
)

$templateIdsRust = New-InputSet -Files @(
    'omoba-template-ids/Cargo.toml',
    'omoba-template-ids/build.rs'
) -Directories @(
    'omoba-template-ids/src'
)

$templateIdsLua = New-InputSet -Files @(
    'scripts/lua_data/templates.lua'
) -Directories @(
    'scripts/lua_data/templates',
    'scripts/lua_data/MVP_1',
    'scripts/lua_data/TD_1',
    'scripts/lua_data/TD_STRESS'
)

$templateIds = Merge-InputSets $templateIdsRust $templateIdsLua

$sim = New-InputSet -Files @(
    'omoba-sim/Cargo.toml',
    'omoba-sim/Cargo.lock'
) -Directories @(
    'omoba-sim/src'
)

$specs = New-InputSet -Files @(
    'specs/Cargo.toml'
) -Directories @(
    'specs/src'
)

$log4rs = New-InputSet -Files @(
    'log4rs/Cargo.toml'
) -Directories @(
    'log4rs/src'
)

$scriptDllInputsDebug = Merge-InputSets $common $scriptAbi $omobaCore $templateIdsRust $sim (New-InputSet -Files @(
    'scripts/Cargo.toml',
    'scripts/Cargo.lock',
    'scripts/base_content/Cargo.toml'
) -Directories @(
    'scripts/base_content/src'
))

$scriptDllInputsRelease = Merge-InputSets $common $scriptAbi $omobaCore $templateIds $sim (New-InputSet -Files @(
    'scripts/Cargo.toml',
    'scripts/Cargo.lock',
    'scripts/base_content/Cargo.toml'
) -Directories @(
    'scripts/base_content/src'
))

$backendInputsDebug = Merge-InputSets $common $scriptAbi $omobaCore $templateIdsRust $sim $specs $log4rs (New-InputSet -Files @(
    'omb/Cargo.toml',
    'omb/Cargo.lock',
    'omb/build.rs'
) -Directories @(
    'omb/src'
))

$backendInputsRelease = Merge-InputSets $common $scriptAbi $omobaCore $templateIds $sim $specs $log4rs (New-InputSet -Files @(
    'omb/Cargo.toml',
    'omb/Cargo.lock',
    'omb/build.rs'
) -Directories @(
    'omb/src'
))

$frontendInputsDebug = Merge-InputSets $common $scriptAbi $omobaCore $templateIdsRust $sim $specs $log4rs (New-InputSet -Files @(
    'omfx/Cargo.toml',
    'omfx/Cargo.lock',
    'omfx/executor/Cargo.toml',
    'omfx/game/Cargo.toml'
) -Directories @(
    'omfx/executor/src',
    'omfx/game/src',
    'third_party/fyrox-impl-1.0.1/src'
))

$frontendInputsRelease = Merge-InputSets $common $scriptAbi $omobaCore $templateIds $sim $specs $log4rs (New-InputSet -Files @(
    'omfx/Cargo.toml',
    'omfx/Cargo.lock',
    'omfx/executor/Cargo.toml',
    'omfx/game/Cargo.toml'
) -Directories @(
    'omfx/executor/src',
    'omfx/game/src',
    'third_party/fyrox-impl-1.0.1/src'
))

$configs = @{
    'script-dll-debug' = @{
        Output = 'scripts/target/debug/base_content.dll'
        Inputs = $scriptDllInputsDebug
        FingerprintDir = 'scripts/target/debug/.fingerprint'
        FingerprintFile = 'lib-base_content.json'
        RequiredFeatures = @('runtime-lua-content')
    }
    'backend-debug' = @{
        Output = 'omb/target/debug/omobab.exe'
        Inputs = $backendInputsDebug
        FingerprintDir = 'omb/target/debug/.fingerprint'
        FingerprintFile = 'bin-omobab.json'
        RequiredFeatures = @('runtime-lua-content')
    }
    'frontend-debug' = @{
        Output = 'omfx/target/debug/executor.exe'
        Inputs = $frontendInputsDebug
        FingerprintDir = 'omfx/target/debug/.fingerprint'
        FingerprintFile = 'bin-executor.json'
        RequiredFeatures = @('runtime-lua-content')
    }
    'script-dll-release' = @{
        Output = 'scripts/target/release/base_content.dll'
        Inputs = $scriptDllInputsRelease
    }
    'backend-release' = @{
        Output = 'omb/target/release/omobab.exe'
        Inputs = $backendInputsRelease
    }
    'frontend-release' = @{
        Output = 'omfx/target/release/executor.exe'
        Inputs = $frontendInputsRelease
    }
}

try {
    if ($Action -eq 'stage-dll') {
        exit (Stage-BaseContentDll -SourceProfile $Profile)
    }

    if ($Action -eq 'stage-backend-spawn') {
        exit (Stage-ReleaseBackendForSpawn)
    }

    if ([string]::IsNullOrWhiteSpace($Artifact)) {
        throw 'Artifact is required when Action is check'
    }

    $configKey = "$Artifact-$Profile"
    if (-not $configs.ContainsKey($configKey)) {
        throw "unknown artifact/profile: $configKey"
    }

    exit (Test-ArtifactFresh -Name $configKey -Config $configs[$configKey])
} catch {
    Write-Error $_.Exception.Message
    exit 2
}
