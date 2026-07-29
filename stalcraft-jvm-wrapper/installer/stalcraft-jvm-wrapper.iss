; STALZONE JVM Wrapper — Inno Setup 6 installer (Windows 10/11 x64)
; Consumes staged files from ..\release\ (produced by scripts\package.ps1)

#define MyAppName "STALZONE JVM Wrapper"
#define MyAppVersion "1.7.3"
#define MyAppPublisher "STALZONE JVM Wrapper"
#define MyAppExeName "stalcraft-jvm-wrapper.exe"
#define MyAppServiceName "service.exe"
#define MyAppId "com.stalcraft.jvm-wrapper"

[Setup]
AppId={{A7C3E91B-4D2F-4E8A-9B1C-6F2E8D0A4B75}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={userappdata}\EXBO\jvm_wrapper
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
; Prefer %AppData%\EXBO\jvm_wrapper so IFEO Debugger matches EXBO layout on any PC.
; User can still change the path in the wizard (Steam / custom folder).
; Keep both exes + examples side by side — required for portable IFEO.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=output
OutputBaseFilename=STALZONE-JVM-Wrapper-Setup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
LicenseFile=license.txt
UninstallDisplayIcon={app}\{#MyAppExeName}
VersionInfoVersion={#MyAppVersion}
VersionInfoProductName={#MyAppName}
SetupLogging=yes
CloseApplications=yes
RestartApplications=no
; Do not register IFEO during Setup by default — path must stay stable.
; Optional task below can call elevated --install after files are in place.

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "ifeoinstall"; Description: "Register IFEO hooks now (requires Administrator / UAC)"; GroupDescription: "Game integration:"; Flags: unchecked

[Files]
; Side-by-side layout required for IFEO Debugger path resolution.
Source: "..\release\stalcraft-jvm-wrapper.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\service.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\examples\*"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
; Never shortcut service.exe — Windows launches it only via IFEO.

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent
; Optional IFEO registration after files are installed to the final path.
Filename: "{app}\{#MyAppExeName}"; Parameters: "--install"; Description: "Register IFEO (Administrator)"; Flags: runascurrentuser postinstall skipifsilent; Tasks: ifeoinstall

[UninstallRun]
; Best-effort IFEO cleanup (prompts UAC if needed). Safe if already uninstalled.
Filename: "{app}\{#MyAppExeName}"; Parameters: "--uninstall"; RunOnceId: "UninstallIfeo"; Flags: runascurrentuser waituntilterminated skipifdoesntexist

[Code]
function InitializeSetup(): Boolean;
begin
  Result := True;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID = wpSelectTasks then
  begin
    if WizardIsTaskSelected('ifeoinstall') then
    begin
      MsgBox(
        'IFEO registration needs Administrator rights.'#13#10 +
        'After Setup finishes, approve the UAC prompt,'#13#10 +
        'or open the app and click INSTALL / VERIFY.',
        mbInformation, MB_OK);
    end;
  end;
end;
