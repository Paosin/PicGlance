#ifndef AppVersion
  #define AppVersion "2.0.0"
#endif
#ifndef AppId
  #define AppId "PicGlance.Desktop"
#endif
#ifndef AppExe
  #define AppExe "..\target\release\picglance.exe"
#endif
#ifndef AppMutexName
  #define AppMutexName "com.picglance.app-sim"
#endif
#ifndef AppIcon
  #define AppIcon "..\src-tauri\icons\icon.ico"
#endif
#ifndef ToolsDir
  #define ToolsDir "..\tools"
#endif

[Setup]
AppId={#AppId}
AppName=PicGlance
AppVersion={#AppVersion}
AppPublisher=PicGlance
DefaultDirName={localappdata}\Programs\PicGlance
DisableDirPage=no
AlwaysShowDirOnReadyPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
ShowLanguageDialog=yes
UninstallLogMode=overwrite
AllowUNCPath=no
ChangesAssociations=yes
AppMutex={#AppMutexName}
UninstallDisplayIcon={app}\PicGlance.exe
SetupIconFile={#AppIcon}
OutputDir=build
OutputBaseFilename=PicGlance-{#AppVersion}-x64-Setup-Offline
Compression=lzma2
SolidCompression=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "zhcn"; MessagesFile: "languages\ChineseSimplified.isl"

[CustomMessages]
en.PortableFolder=Choose an installation folder separate from your portable copy.
zhcn.PortableFolder=请选择与便携版不同的安装目录。
en.UnsafeFolder=Setup or uninstall cannot continue because a PicGlance path is redirected or could not be checked.
zhcn.UnsafeFolder=PicGlance 的目录路径包含重定向或无法验证，安装或卸载已停止。
en.CreateParentFailed=Could not create the parent folder for the installation.
zhcn.CreateParentFailed=无法创建安装目录的上级目录。
en.RuntimeFailed=The included Microsoft Edge WebView2 Runtime could not be installed.
zhcn.RuntimeFailed=无法安装安装包内附的 Microsoft Edge WebView2 运行时。
en.ContextMenuFailed=PicGlance could not register its Explorer context menu.
zhcn.ContextMenuFailed=无法注册「用 PicGlance 打开」右键菜单。

[Files]
Source: "{#AppExe}"; DestDir: "{app}"; DestName: "PicGlance.exe"; Flags: ignoreversion
Source: "{#ToolsDir}\pngquant.exe"; DestDir: "{app}\tools"; Flags: ignoreversion
Source: "{#ToolsDir}\gifsicle.exe"; DestDir: "{app}\tools"; Flags: ignoreversion
Source: "{#ToolsDir}\README.md"; DestDir: "{app}\tools"; Flags: ignoreversion
Source: "third-party\*"; DestDir: "{app}\tools\third-party"; Flags: ignoreversion
Source: "{#OfflineRuntime}"; DestDir: "{tmp}"; DestName: "WebView2Setup.exe"; Flags: dontcopy

[Run]
Filename: "{app}\PicGlance.exe"; Description: "{cm:LaunchProgram,PicGlance}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\PicGlance.exe"; Parameters: "--desktop-uninstall"; Flags: runhidden waituntilterminated; RunOnceId: "RemoveDesktopIntegration"

[UninstallDelete]
Type: files; Name: "{app}\log\picglance.log"
Type: files; Name: "{app}\log\picglance.log.1"
Type: files; Name: "{app}\log\picglance.log.2"
Type: files; Name: "{app}\log\picglance.log.3"
Type: dirifempty; Name: "{app}\log"

[Code]
const
  EdgeClients = 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
  ReparsePoint = $400;
  InvalidAttributes = $FFFFFFFF;

function GetFileAttributes(Path: String): LongWord;
  external 'GetFileAttributesW@kernel32.dll stdcall';

function OrdinaryPath(Path: String): Boolean;
var
  Parent: String;
  Attributes: LongWord;
  ErrorCode: Integer;
begin
  Result := False;
  if (Length(Path) < 3) or (Copy(Path, 2, 2) <> ':\') then exit;
  Parent := ExtractFileDir(Path);
  if (Length(Parent) > 3) and (CompareText(Parent, Path) <> 0) then
    if not OrdinaryPath(Parent) then exit;
  Attributes := GetFileAttributes(Path);
  if Attributes = InvalidAttributes then begin
    ErrorCode := DLLGetLastError;
    Result := (ErrorCode = 2) or (ErrorCode = 3);
  end else begin
    Result := (Attributes and ReparsePoint) = 0;
  end;
end;

function SafeAppPaths(AppDir: String): Boolean;
begin
  Result := (Length(RemoveBackslashUnlessRoot(AppDir)) > 3) and
            OrdinaryPath(AddBackslash(AppDir) + 'tools\third-party') and
            OrdinaryPath(AddBackslash(AppDir) + 'log') and
            OrdinaryPath(AddBackslash(AppDir) + 'PicGlance.exe');
end;

function SafeUninstallPaths: Boolean;
var
  AppDir, Uninstaller: String;
begin
  AppDir := ExpandConstant('{app}');
  Uninstaller := ExpandConstant('{uninstallexe}');
  Result := (CompareText(RemoveBackslashUnlessRoot(AppDir), ExtractFileDir(Uninstaller)) = 0) and
            SafeAppPaths(AppDir) and OrdinaryPath(Uninstaller) and
            OrdinaryPath(ChangeFileExt(Uninstaller, '.dat'));
end;

function InitializeUninstall: Boolean;
begin
  Result := SafeUninstallPaths;
  if not Result then
    SuppressibleMsgBox(CustomMessage('UnsafeFolder'), mbError, MB_OK, IDOK);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if (CurUninstallStep = usUninstall) and not SafeUninstallPaths then
    RaiseException(CustomMessage('UnsafeFolder'));
end;

function RuntimePresent: Boolean;
var
  Version: String;
begin
  Result := (RegQueryStringValue(HKLM32, EdgeClients, 'pv', Version) and (Version <> '') and (Version <> '0.0.0.0')) or
            (RegQueryStringValue(HKCU, EdgeClients, 'pv', Version) and (Version <> '') and (Version <> '0.0.0.0'));
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ExitCode: Integer;
  AppDir: String;
begin
  Result := '';
  AppDir := ExpandConstant('{app}');
  if not SafeAppPaths(AppDir) then begin
    Result := CustomMessage('UnsafeFolder');
    exit;
  end;
  if FileExists(ExpandConstant('{app}\portable.marker')) then begin
    Result := CustomMessage('PortableFolder');
    exit;
  end;
  if not RuntimePresent then begin
    ExtractTemporaryFile('WebView2Setup.exe');
    if not Exec(ExpandConstant('{tmp}\WebView2Setup.exe'), '/silent /install', '',
      SW_HIDE, ewWaitUntilTerminated, ExitCode) or (ExitCode <> 0) or not RuntimePresent then begin
      Result := CustomMessage('RuntimeFailed');
      exit;
    end;
  end;
  { Inno Setup records directories it creates. Create the parent separately
    so it cannot enter the uninstall log. }
  if not ForceDirectories(ExtractFileDir(RemoveBackslashUnlessRoot(AppDir))) then
    Result := CustomMessage('CreateParentFailed');
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ExitCode: Integer;
begin
  if CurStep <> ssPostInstall then exit;
  if not Exec(ExpandConstant('{app}\PicGlance.exe'), '--desktop-install',
    '', SW_HIDE, ewWaitUntilTerminated, ExitCode) or (ExitCode <> 0) then
    RaiseException(CustomMessage('ContextMenuFailed'));
end;
