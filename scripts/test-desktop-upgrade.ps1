[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$InstallDirectory,
    [Parameter(Mandatory)][string]$UpgradeInstaller,
    [string]$UpgradeVersion = '2.0.1',
    [string]$UpgradeExe
)
$ErrorActionPreference = 'Stop'
$dir = (Resolve-Path -LiteralPath $InstallDirectory).Path
$exe = Join-Path $dir 'PicGlance.exe'
$payload = 'pngquant.exe', 'gifsicle.exe', 'README.md'
$uninstaller = Join-Path $dir 'unins000.exe'
$settings = Join-Path $env:APPDATA 'PicGlance/settings.ini'
$associations = 'HKCU:/Software/PicGlance/Desktop/Capabilities/FileAssociations'
$registration = 'HKCU:/Software/PicGlance/Desktop'
$classes = 'HKCU:/Software/Classes'
$uninstallKey = 'HKCU:/Software/Microsoft/Windows/CurrentVersion/Uninstall/PicGlance.Desktop_is1'
$registered = 'HKCU:/Software/RegisteredApplications'

function Invoke-TestProcess($File, [string[]]$Arguments) {
    $process = Start-Process -FilePath $File -ArgumentList $Arguments -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "$File exited with $($process.ExitCode)" }
}

try {
    foreach ($file in $payload) {
        $expected = Get-FileHash -LiteralPath (Join-Path $PSScriptRoot "../tools/$file")
        $actual = Get-FileHash -LiteralPath (Join-Path $dir "tools/$file")
        if ($expected.Hash -ne $actual.Hash) { throw "Installer payload mismatch for $file" }
    }
    if (Test-Path "$classes/PicGlance.Desktop.png") { throw 'Setup registered a PNG handler.' }
    $before = (Get-FileHash -LiteralPath $settings).Hash

    # Model a previous install with PNG and a retired AVIF registration.
    Invoke-TestProcess $exe @('--desktop-install', 'png')
    Set-ItemProperty -LiteralPath $registration -Name Supported -Value 'png,avif'
    New-Item -Path "$classes/PicGlance.Desktop.avif" -Force | Out-Null
    New-Item -Path "$classes/.avif/OpenWithProgids" -Force | Out-Null
    New-Item -Path "$classes/SystemFileAssociations/.avif/shell/PicGlance.Desktop" -Force | Out-Null
    New-ItemProperty -LiteralPath "$classes/.avif/OpenWithProgids" -Name 'PicGlance.Desktop.avif' -Value '' -Force | Out-Null
    New-ItemProperty -LiteralPath $associations -Name '.avif' -Value 'PicGlance.Desktop.avif' -Force | Out-Null

    Invoke-TestProcess (Resolve-Path -LiteralPath $UpgradeInstaller).Path @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART')
    if ((Get-ItemProperty -LiteralPath $uninstallKey).DisplayVersion -ne $UpgradeVersion) { throw 'Wrong installed version.' }
    if ($UpgradeExe -and (Get-FileHash -LiteralPath $exe).Hash -ne (Get-FileHash -LiteralPath $UpgradeExe).Hash) {
        throw 'Upgrade did not deploy the expected executable.'
    }
    if ((Get-FileHash -LiteralPath $settings).Hash -ne $before) { throw 'Upgrade changed the settings file.' }
    if ((Test-Path -LiteralPath $associations) -or
        (Test-Path "$classes/PicGlance.Desktop.png") -or
        ((Get-Item "$classes/.png/OpenWithProgids").GetValueNames() -contains 'PicGlance.Desktop.png') -or
        ((Get-Item -LiteralPath $registered).GetValueNames() -contains 'PicGlance') -or
        (Test-Path "$classes/PicGlance.Desktop.avif") -or
        (Test-Path "$classes/SystemFileAssociations/.avif/shell/PicGlance.Desktop") -or
        (Get-Item "$classes/.avif/OpenWithProgids").GetValueNames() -contains 'PicGlance.Desktop.avif') {
        throw 'Upgrade left previous file associations.'
    }
    $verb = 'Registry::HKEY_CURRENT_USER\Software\Classes\*\shell\PicGlance.Desktop'
    if (-not (Test-Path -LiteralPath "$verb\command")) {
        throw 'Upgrade lost the traditional context menu.'
    }
    Write-Output 'Upgrade preserved settings and the Explorer verb, and removed legacy file associations.'
} finally {
    if (Test-Path -LiteralPath $uninstaller) {
        Invoke-TestProcess $uninstaller @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART')
    }
}
foreach ($path in @($exe, $registration, $uninstallKey, "$classes/PicGlance.Desktop.jpg",
    'Registry::HKEY_CURRENT_USER\Software\Classes\*\shell\PicGlance.Desktop') +
    ($payload | ForEach-Object { Join-Path $dir "tools/$_" })) {
    if (Test-Path -LiteralPath $path) { throw "Uninstall left $path" }
}
if ((Get-Item -LiteralPath $registered).GetValueNames() -contains 'PicGlance') { throw 'Uninstall left the default apps registration.' }
if ((Get-FileHash -LiteralPath $settings).Hash -ne $before) { throw 'Uninstall changed the user settings.' }
Write-Output 'Uninstall removed the app and context menu while preserving user settings.'
