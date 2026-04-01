; SELFIDEX v3.0 Installer Script for Inno Setup
; Instalador SEGURO que NO elimina el PATH de Windows
; Requiere: Inno Setup 6+ (https://jrsoftware.org/isinfo.php)

#define MyAppName "SELFIDEX"
#define MyAppVersion "3.0.0"
#define MyAppPublisher "SELFIDEX"
#define MyAppURL "https://github.com/zerumen82/SELFIDX"
#define MyAppExeName "selfidx.exe"

[Setup]
AppId={{F7A8B2C4-D5E6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=SELFIDEX-Setup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
DisableWelcomePage=no
DisableProgramGroupPage=yes
WizardResizable=no

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addpath"; Description: "Agregar SELFIDEX al PATH (recomendado)"; GroupDescription: "Opciones:"; Flags: checked
Name: "backup"; Description: "Crear backup del PATH actual"; GroupDescription: "Seguridad:"; Flags: checked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "install.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "uninstall.bat"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Parameters: "--help"; Description: "Ver ayuda de {#MyAppName}"; Flags: postinstall skipifsilent unchecked

[Code]
var
  BackupPage: TWizardPage;
  BackupLabel: TLabel;
  BackupMemo: TMemo;
  BackupPath: String;

// ============================================================================
// BACKUP DEL PATH ANTES DE MODIFICAR
// ============================================================================
procedure CreatePathBackup;
var
  CurrentPath: String;
  BackupFile: String;
  BackupDir: String;
begin
  // Obtener PATH actual
  CurrentPath := ExpandConstant('{reg:HKCU\Environment,Path}');
  
  // Crear directorio de backups
  BackupDir := ExpandConstant('{localappdata}\selfidx\backups');
  CreateDir(BackupDir);
  
  // Nombre del archivo con fecha
  BackupFile := BackupDir + '\path_backup_' + GetDateTimeString('yyyymmdd_hhnnss', '-', '') + '.txt';
  
  // Guardar PATH en archivo
  SaveStringToFile(BackupFile, CurrentPath, False);
  
  // Mostrar en el memo
  BackupMemo.Lines.Add('Backup creado: ' + BackupFile);
  BackupMemo.Lines.Add('PATH actual (' + IntToStr(Length(CurrentPath)) + ' caracteres):');
  BackupMemo.Lines.Add(CurrentPath);
  BackupMemo.Lines.Add('');
  
  BackupPath := BackupFile;
end;

// ============================================================================
// VERIFICAR SI SELFIDEX YA ESTÁ EN EL PATH
// ============================================================================
function IsSelfIdxInPath(): Boolean;
var
  CurrentPath: String;
  SelfIdxPath: String;
begin
  CurrentPath := ExpandConstant('{reg:HKCU\Environment,Path}');
  SelfIdxPath := ExpandConstant('{localappdata}\selfidx');
  
  Result := Pos(SelfIdxPath, CurrentPath) > 0;
end;

// ============================================================================
// AGREGAR AL PATH DE FORMA SEGURA (SIN ELIMINAR EL EXISTENTE)
// ============================================================================
function AddToPathSafe(const PathToAdd: String): Boolean;
var
  CurrentPath: String;
  NewPath: String;
begin
  Result := False;
  
  try
    // Obtener PATH actual
    CurrentPath := ExpandConstant('{reg:HKCU\Environment,Path}');
    
    // Verificar si ya está en el PATH
    if Pos(PathToAdd, CurrentPath) > 0 then
    begin
      Log('SELFIDEX ya está en el PATH, no se necesita modificar');
      Result := True;
      Exit;
    end;
    
    // AGREGAR al final (NO reemplazar)
    if Length(CurrentPath) > 0 then
      NewPath := CurrentPath + ';' + PathToAdd
    else
      NewPath := PathToAdd;
    
    // Guardar nuevo PATH
    RegWriteStringValue(HKCU, 'Environment', 'Path', NewPath);
    
    Log('PATH actualizado correctamente');
    Log('Longitud anterior: ' + IntToStr(Length(CurrentPath)));
    Log('Longitud nueva: ' + IntToStr(Length(NewPath)));
    
    Result := True;
  except
    Log('ERROR al modificar el PATH: ' + GetExceptionMessage);
    Result := False;
  end;
end;

// ============================================================================
// ELIMINAR DEL PATH (DESINSTALACIÓN)
// ============================================================================
procedure RemoveFromPath(const PathToRemove: String);
var
  CurrentPath: String;
  NewPath: String;
  PathParts: TArrayOfString;
  I: Integer;
begin
  try
    CurrentPath := ExpandConstant('{reg:HKCU\Environment,Path}');
    
    // Dividir PATH en partes
    PathParts := StringSplit(CurrentPath, ';');
    NewPath := '';
    
    // Reconstruir PATH sin la ruta a eliminar
    for I := 0 to GetArrayLength(PathParts) - 1 do
    begin
      if Pos(PathToRemove, PathParts[I]) = 0 then
      begin
        if Length(NewPath) > 0 then
          NewPath := NewPath + ';' + PathParts[I]
        else
          NewPath := PathParts[I];
      end;
    end;
    
    // Guardar nuevo PATH
    RegWriteStringValue(HKCU, 'Environment', 'Path', NewPath);
    
    Log('SELFIDEX eliminado del PATH');
  except
    Log('ERROR al eliminar del PATH: ' + GetExceptionMessage);
  end;
end;

// ============================================================================
// PÁGINA DE BACKUP
// ============================================================================
procedure InitializeWizard;
begin
  BackupPage := CreateCustomPage(wpInstalling, 'Backup del PATH', 'Creando copia de seguridad del PATH de Windows...');
  
  BackupLabel := TLabel.Create(WizardForm);
  BackupLabel.Parent := BackupPage.Surface;
  BackupLabel.Caption:='Se creará un backup del PATH actual antes de instalar:';
  BackupLabel.Left := 10;
  BackupLabel.Top := 10;
  BackupLabel.Width := 500;
  BackupLabel.Height := 20;
  
  BackupMemo := TMemo.Create(WizardForm);
  BackupMemo.Parent := BackupPage.Surface;
  BackupMemo.Left := 10;
  BackupMemo.Top := 35;
  BackupMemo.Width := 500;
  BackupMemo.Height := 200;
  BackupMemo.ReadOnly := True;
  BackupMemo.ScrollBars := ssVertical;
end;

// ============================================================================
// EJECUTAR BACKUP DURANTE LA INSTALACIÓN
// ============================================================================
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
  begin
    // Crear backup antes de instalar
    if WizardIsTaskSelected('backup') then
    begin
      BackupMemo.Lines.Clear;
      BackupMemo.Lines.Add('Creando backup del PATH...');
      CreatePathBackup;
    end;
  end;
end;

// ============================================================================
// AGREGAR AL PATH DESPUÉS DE INSTALAR
// ============================================================================
procedure CurInstallProgressChanged(CurInstallProgressChanged, CurProgress: Int64);
begin
  // No hacer nada aquí, solo para mostrar progreso
end;

// ============================================================================
// FINALIZAR INSTALACIÓN
// ============================================================================
procedure CurStepChangedInstall(CurStep: TSetupStep);
var
  SelfIdxPath: String;
begin
  if CurStep = ssPostInstall then
  begin
    // Agregar al PATH si se seleccionó la tarea
    if WizardIsTaskSelected('addpath') then
    begin
      SelfIdxPath := ExpandConstant('{app}');
      
      if not AddToPathSafe(SelfIdxPath) then
      begin
        MsgBox('Error al agregar SELFIDEX al PATH. Puedes hacerlo manualmente después.', mbError, MB_OK);
      end;
    end;
  end;
end;

// ============================================================================
// DESINSTALACIÓN - ELIMINAR DEL PATH
// ============================================================================
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  SelfIdxPath: String;
begin
  if CurUninstallStep = usUninstall then
  begin
    SelfIdxPath := ExpandConstant('{app}');
    RemoveFromPath(SelfIdxPath);
    
    Log('Desinstalación completada');
    Log('PATH restaurado (SELFIDEX eliminado)');
  end;
end;
