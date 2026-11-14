[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Installer,
    [string]$AppId = 'PicGlance.Desktop',
    [switch]$FileBoundaryOnly
)

$ErrorActionPreference = 'Stop'
$installerPath = (Resolve-Path -LiteralPath $Installer).Path
$uninstallKey = "HKCU:/Software/Microsoft/Windows/CurrentVersion/Uninstall/${AppId}_is1"
$registration = 'HKCU:/Software/PicGlance/Desktop'
$classes = 'Registry::HKEY_CURRENT_USER\Software\Classes'
$verb = "$classes\*\shell\PicGlance.Desktop"
if ((Test-Path -LiteralPath $uninstallKey) -or (Get-Process PicGlance -ErrorAction SilentlyContinue) -or
    (!$FileBoundaryOnly -and ((Test-Path -LiteralPath $registration) -or (Test-Path -LiteralPath $verb)))) {
    throw 'Run in a Windows profile without an existing or running PicGlance installation.'
}

$run = (New-Item -ItemType Directory -Path (Join-Path (Split-Path $installerPath) ('boundary-' + [guid]::NewGuid()))).FullName
$parent = Join-Path $run 'new-parent'
$app = Join-Path $parent 'PicGlance'
$outside = (New-Item -ItemType Directory -Path (Join-Path $run 'unrelated')).FullName
$uninstaller = Join-Path $app 'unins000.exe'
$junction = Join-Path $app 'log'
$sentinel = Join-Path $outside 'picglance.log'
$default = "$classes\.png"
$defaultBefore = if (Test-Path -LiteralPath $default) { (Get-Item -LiteralPath $default).GetValue('') }
Set-Content -LiteralPath $sentinel -Value 'outside install directory'
$originalHash = (Get-FileHash -LiteralPath $sentinel).Hash

function Invoke-Installer([string]$File, [string]$Options = '') {
    $process = Start-Process -FilePath $File -ArgumentList "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART $Options" -WindowStyle Hidden -Wait -PassThru
    return $process.ExitCode
}

try {
    $code = Invoke-Installer $installerPath "/LANG=zhcn /DIR=""$app"" /LOG=""$run\install.log"""
    if ($code -ne 0) { throw "Install failed with exit code $code. See $run\install.log." }
    if (!$FileBoundaryOnly -and (!(Test-Path -LiteralPath "$verb\command") -or
        (Test-Path -LiteralPath "$registration/Capabilities") -or
        (Test-Path -LiteralPath "$classes\PicGlance.Desktop.png"))) {
        throw 'Setup must register the Explorer verb without registering a default app or format.'
    }
    $defaultAfter = if (Test-Path -LiteralPath $default) { (Get-Item -LiteralPath $default).GetValue('') }
    if ($defaultBefore -cne $defaultAfter) { throw 'Setup changed the PNG handler.' }

    New-Item -ItemType Junction -Path $junction -Target $outside | Out-Null
    $code = Invoke-Installer $uninstaller "/LOG=""$run\blocked.log"""
    if ($code -eq 0 -or !(Test-Path -LiteralPath "$app\PicGlance.exe") -or
        (Get-FileHash -LiteralPath $sentinel).Hash -ne $originalHash) {
        throw 'The uninstaller must stop when a log path leads to a different folder.'
    }
    [IO.Directory]::Delete($junction)

    $code = Invoke-Installer $uninstaller "/LOG=""$run\uninstall.log"""
    $deletions = Select-String -LiteralPath "$run\uninstall.log" -Pattern 'Deleting (?:file|directory): (.+)$'
    $escaped = $deletions | Where-Object {
        $path = $_.Matches[0].Groups[1].Value
        $path -ne $app -and !$path.StartsWith("$app\", [StringComparison]::OrdinalIgnoreCase)
    }
    if (!$deletions -or $escaped) { throw 'The uninstall log must contain deletions only within the installation directory.' }
    if ($code -ne 0 -or (Test-Path -LiteralPath "$app\PicGlance.exe") -or
        (!$FileBoundaryOnly -and (Test-Path -LiteralPath $verb)) -or (Test-Path -LiteralPath $uninstallKey) -or
        !(Test-Path -LiteralPath $parent) -or (Get-FileHash -LiteralPath $sentinel).Hash -ne $originalHash) {
        throw 'Uninstall must clean up PicGlance while preserving the parent folder and outside file.'
    }
    Write-Output "Installed in Chinese, blocked a junction, and uninstalled within the installation directory. Logs: $run"
} finally {
    if ((Get-Item -LiteralPath $junction -ErrorAction SilentlyContinue).LinkType -eq 'Junction') {
        [IO.Directory]::Delete($junction)
    }
    if (Test-Path -LiteralPath $uninstaller) {
        $null = Invoke-Installer $uninstaller "/LOG=""$run\cleanup.log"""
    }
}
