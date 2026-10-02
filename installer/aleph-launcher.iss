; Aleph Minecraft Client Launcher — Windows installer (CONCEPT P11).
; Inno Setup 6 script: checks .NET/VC++ prerequisites, installs the launcher,
; offers Desktop shortcut and autostart, asks about data on uninstall.

#define AppName "Aleph Launcher"
#define AppVersion "0.1.0"
#define AppPublisher "Aleph Team"
#define AppURL "https://github.com/KurumaOfficial/Aleph-Minecraft-Client-Launcher"
#define AppExe "amc-launcher.exe"

[Setup]
AppId={{A17C4E11-9B2A-4E5C-9F1A-ALEPLAUNCHER}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
DefaultDirName={autopf}\AlephLauncher
DefaultGroupName={#AppName}
OutputBaseFilename=AlephLauncher-Setup-{#AppVersion}
Compression=lzma2/max
SolidCompression=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayName={#AppName}
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "ukrainian"; MessagesFile: "compiler:Languages\Ukrainian.isl"

[Tasks]
Name: "desktopicon"; Description: "Desktop shortcut"; GroupDescription: "Shortcuts:";
Name: "autostart"; Description: "Start with Windows"; GroupDescription: "Startup:";

[Files]
Source: "target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "AlephLauncher"; ValueData: """{app}\{#AppExe}"""; Tasks: autostart; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#AppExe}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent

[Code]
function IsDotNetAvailable(): Boolean;
var
  Names: TArrayOfString;
  I: Integer;
begin
  Result := False;
  if RegGetSubkeyNames(HKLM, 'SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full', Names) then
    Result := True;
end;

function InitializeSetup(): Boolean;
begin
  Result := True;
  if not IsDotNetAvailable() then
  begin
    if MsgBox('Microsoft .NET Framework 4.8 was not detected. The launcher itself does not need it, but some Java distributions do. Continue anyway?', mbConfirmation, MB_YESNO) = IDNO then
      Result := False;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DataDir, Answer: Integer;
begin
  if CurUninstallStep = usUninstall then
  begin
    DataDir := ExpandConstant('{userappdata}\AlephLauncher');
    if DirExists(DataDir) then
    begin
      Answer := MsgBox('Delete launcher data too (instances, worlds, settings)?', mbConfirmation, MB_YESNO);
      if Answer = IDYES then
        DelTree(DataDir, True, True, True);
    end;
  end;
end;
