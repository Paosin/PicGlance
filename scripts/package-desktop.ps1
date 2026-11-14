[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Exe,
    [Parameter(Mandatory)][string]$Compiler,
    [Parameter(Mandatory)][string]$WebView2StandaloneInstaller,
    [ValidatePattern('^\d+\.\d+\.\d+([.-][0-9A-Za-z.-]+)?$')][string]$Version = '2.0.0',
    [string]$AppMutex = 'com.picglance.app-sim',
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '../installer/build')
)

$ErrorActionPreference = 'Stop'
$exePath = (Resolve-Path -LiteralPath $Exe).Path
$compilerPath = (Resolve-Path -LiteralPath $Compiler).Path
$runtimePath = (Resolve-Path -LiteralPath $WebView2StandaloneInstaller).Path
$signature = Get-AuthenticodeSignature -LiteralPath $runtimePath
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'CN=Microsoft Corporation(?:,|$)') {
    throw 'Expected a valid Microsoft-signed WebView2 installer.'
}
$installerDir = Join-Path $PSScriptRoot '../installer'
$output = (New-Item -ItemType Directory -Force -Path $OutputDirectory).FullName
$icon = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../src-tauri/icons/icon.ico')).Path
$tools = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../tools')).Path
$encoders = 'pngquant.exe', 'gifsicle.exe'
$toolFiles = $encoders + 'README.md'
$thirdPartyDir = Join-Path $installerDir 'third-party'
$thirdPartyFiles = Get-ChildItem -LiteralPath $thirdPartyDir -File
foreach ($file in $toolFiles) {
    if (-not (Test-Path -LiteralPath (Join-Path $tools $file) -PathType Leaf)) {
        throw "Missing compression payload file $file."
    }
}
& $compilerPath "/O$output" "/DAppExe=$exePath" "/DAppVersion=$Version" "/DAppMutexName=$AppMutex" "/DAppIcon=$icon" "/DToolsDir=$tools" "/DOfflineRuntime=$runtimePath" (Join-Path $installerDir 'PicGlance.iss')
if ($LASTEXITCODE -ne 0) { throw "Inno Setup returned $LASTEXITCODE." }

Add-Type -AssemblyName System.IO.Compression.FileSystem
$zipPath = Join-Path $output "PicGlance-$Version-x64-Portable.zip"
if (Test-Path -LiteralPath $zipPath) { Remove-Item -LiteralPath $zipPath }
$archive = [IO.Compression.ZipFile]::Open($zipPath, [IO.Compression.ZipArchiveMode]::Create)
try {
    [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $exePath, 'PicGlance.exe', [IO.Compression.CompressionLevel]::Optimal) | Out-Null
    foreach ($file in $toolFiles) {
        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, (Join-Path $tools $file), "tools/$file", [IO.Compression.CompressionLevel]::Optimal) | Out-Null
    }
    foreach ($file in $thirdPartyFiles) {
        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $file.FullName, "tools/third-party/$($file.Name)", [IO.Compression.CompressionLevel]::Optimal) | Out-Null
    }
    $archive.CreateEntry('portable.marker') | Out-Null
} finally {
    $archive.Dispose()
}
Get-Item -LiteralPath $zipPath
