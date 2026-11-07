#ifndef AppVersion
  #error AppVersion must be provided by package-windows.ps1
#endif
#ifndef SourceDir
  #error SourceDir must be provided by package-windows.ps1
#endif
#ifndef ArtifactDir
  #error ArtifactDir must be provided by package-windows.ps1
#endif

[Setup]
AppId={{9206E8A0-A3F3-4F93-8717-6FF7D87BB584}
AppName=SmithCraft
AppVersion={#AppVersion}
AppPublisher=SmithCraft
DefaultDirName={localappdata}\Programs\SmithCraft
DefaultGroupName=SmithCraft
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
DisableProgramGroupPage=yes
OutputDir={#ArtifactDir}
OutputBaseFilename=SmithCraft-{#AppVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\SmithCraft.exe
LicenseFile={#SourceDir}\LICENSE
CloseApplications=yes

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Icons]
Name: "{group}\SmithCraft"; Filename: "{app}\SmithCraft.exe"
Name: "{userdesktop}\SmithCraft"; Filename: "{app}\SmithCraft.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\SmithCraft.exe"; Description: "Launch SmithCraft"; Flags: nowait postinstall skipifsilent
