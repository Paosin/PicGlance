[CmdletBinding()]
param([string]$Artifacts = (Join-Path $PSScriptRoot '../installer/build/production'))

$ErrorActionPreference = 'Stop'
$artifactsPath = (Resolve-Path -LiteralPath $Artifacts).Path
$run = Join-Path $artifactsPath ('validation-' + [guid]::NewGuid().ToString('N'))
$installed = Join-Path $run 'installed'
$portable = Join-Path $run 'portable'
$registry = 'HKCU:/Software/PicGlance/Desktop'
$uninstallKey = 'HKCU:/Software/Microsoft/Windows/CurrentVersion/Uninstall/PicGlance.Desktop_is1'
$thirdParty = Get-ChildItem -LiteralPath (Join-Path $PSScriptRoot '../installer/third-party') -File

if ((Test-Path $registry) -or (Test-Path $uninstallKey)) {
    throw 'Run in a Windows profile without an existing PicGlance install.'
}
New-Item -ItemType Directory -Path $run | Out-Null
$fixture = Join-Path $run ('PicGlance-test-' + [char]0x4e2d + ' image.png')
$second = Join-Path $run ('PicGlance-test-' + [char]0x6587 + ' second.png')
python -c "import sys; from PIL import Image; Image.new('RGB',(40,30),'red').save(sys.argv[1]); Image.new('RGB',(40,30),'blue').save(sys.argv[2])" $fixture $second
if ($LASTEXITCODE -ne 0) { throw 'Could not generate image fixtures.' }

function Invoke-Inno([string]$File, [string]$Additional = '') {
    $process = Start-Process -FilePath $File -ArgumentList "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART $Additional" -WindowStyle Hidden -PassThru
    if (-not $process.WaitForExit(180000)) {
        & taskkill.exe /PID $process.Id /T /F | Out-Null
        throw "$File did not finish within three minutes."
    }
    if ($process.ExitCode -ne 0) { throw "$File exited with $($process.ExitCode)." }
}

function Uninstall([string]$Directory) {
    $uninstaller = Join-Path $Directory 'unins000.exe'
    if (Test-Path -LiteralPath $uninstaller) { Invoke-Inno $uninstaller }
}

function Check-Payload([string]$Directory) {
    foreach ($file in $thirdParty) {
        if ((Get-FileHash -LiteralPath (Join-Path $Directory "tools/third-party/$($file.Name)")).Hash -ne
            (Get-FileHash -LiteralPath $file.FullName).Hash) {
            throw "Third-party materials differ for $($file.Name)."
        }
    }
    foreach ($file in 'tools/pngquant.exe', 'tools/gifsicle.exe', 'tools/README.md') {
        if ((Get-FileHash -LiteralPath (Join-Path $Directory $file)).Hash -ne
            (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot "../$file")).Hash) {
            throw "Payload hash differs for $file."
        }
    }
    if ((Get-FileHash -LiteralPath (Join-Path $Directory 'PicGlance.exe')).Hash -ne
        (Get-FileHash -LiteralPath (Join-Path $artifactsPath 'PicGlance.exe')).Hash) {
        throw 'Payload executable differs from the 2.0.0 build.'
    }
}

try {
    Invoke-Inno (Join-Path $artifactsPath 'PicGlance-2.0.0-x64-Setup-Offline.exe') "/DIR=""$installed"" /LOG=""$run/install.log"""
    Check-Payload $installed
    $zip = Join-Path $artifactsPath 'PicGlance-2.0.0-x64-Portable.zip'
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($zip)
    try {
        $expected = 'PicGlance.exe', 'portable.marker', 'tools/pngquant.exe', 'tools/gifsicle.exe', 'tools/README.md'
        $expected += $thirdParty | ForEach-Object { "tools/third-party/$($_.Name)" }
        $entries = $archive.Entries.FullName
        if ($entries.Count -ne $expected.Count -or (Compare-Object $expected $entries -CaseSensitive)) {
            throw 'Unexpected portable archive entries.'
        }
    } finally { $archive.Dispose() }
    Expand-Archive -LiteralPath $zip -DestinationPath $portable
    Check-Payload $portable
    node (Join-Path $PSScriptRoot 'test-desktop-host.mjs') $installed $portable $fixture $second production
    if ($LASTEXITCODE -ne 0) { throw 'Production smoke failed.' }
    $upgrade = @{
        InstallDirectory = $installed
        UpgradeInstaller = Join-Path $artifactsPath 'PicGlance-2.0.1-x64-Setup-Offline.exe'
        UpgradeExe = Join-Path $artifactsPath 'PicGlance-upgrade.exe'
    }
    & (Join-Path $PSScriptRoot 'test-desktop-upgrade.ps1') @upgrade
} finally {
    Uninstall $installed
}

if ((Test-Path $registry) -or (Test-Path $uninstallKey) -or (Test-Path "$installed/PicGlance.exe")) {
    throw 'Offline uninstall left app registration or executable.'
}
