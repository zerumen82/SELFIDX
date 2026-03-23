; SELFIDEX v3.0 Installer Script for Inno Setup
; Instalador moderno que no requiere permisos de administrador

#define MyAppName "SELFIDEX"
#define MyAppVersion "3.0.0"
#define MyAppPublisher "SELFIDEX"
#define MyAppURL "https://github.com/selfidx/selfidx"
#define MyAppExeName "selfidx.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
; Use user's local directory instead of Program Files (no admin needed)
DefaultDirName={localappdata}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=
OutputDir=..\dist
OutputBaseFilename=SELFIDEX-Setup-{#MyAppVersion}
SetupIconFile=
Compression=lzma
SolidCompression=yes
WizardStyle=modern
; No admin privileges required
PrivilegesRequired=lowest
; Show welcome page
DisableWelcomePage=no
; Show components page
DisableProgramGroupPage=yes

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addpath"; Description: "Agregar al PATH del usuario"; GroupDescription: "Opciones adicionales:"; Flags: checked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Run the install command to add to user PATH
Filename: "{app}\{#MyAppExeName}"; Parameters: "--install"; Description: "Agregar al PATH del usuario"; Flags: postinstall skipifsilent; Tasks: addpath
; Option to run SELFIDEX after installation
Filename: "{app}\{#MyAppExeName}"; Description: "Ejecutar SELFIDEX"; Flags: postinstall nowait skipifsilent unchecked

[Code]
// Custom uninstall cleanup
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // Remove from user PATH if it was added
    if MsgBox('¿Deseas eliminar SELFIDEX del PATH del usuario?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      // Note: The actual PATH removal is handled by the uninstaller
      // This is just a confirmation prompt
    end;
  end;
end;

// Welcome page customization
procedure InitializeWizard;
begin
  WizardForm.WelcomeLabel1.Caption := 'Bienvenido al instalador de SELFIDEX v3.0';
  WizardForm.WelcomeLabel2.Caption := 'Este asistente te guiará a través de la instalación de SELFIDEX, una terminal integrada con capacidades de IA.' + #13#10 + #13#10 + 'SELFIDEX te permite:' + #13#10 + '• Ejecutar comandos del sistema' + #13#10 + '• Chatear con IA usando Jan.ai' + #13#10 + '• Usar un agente de programación autónomo' + #13#10 + '• Navegar y editar archivos' + #13#10 + #13#10 + 'Haz clic en Siguiente para continuar.';
end;

// Finished page customization
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    WizardForm.FinishedLabel.Caption := 'SELFIDEX v3.0 se ha instalado correctamente.' + #13#10 + #13#10 + 'Para usar SELFIDEX:' + #13#10 + '1. Abre una NUEVA terminal (CMD o PowerShell)' + #13#10 + '2. Ejecuta: selfidx --help' + #13#10 + #13#10 + 'Nota: Si el comando no funciona, reinicia tu terminal o tu PC.';
  end;
end;
