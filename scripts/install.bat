@echo off
REM ============================================================================
REM SELFIDEX v3.0 - Instalador Seguro para Windows
REM ============================================================================
REM Características:
REM - Backup automático del PATH antes de modificar
REM - No elimina el PATH existente
REM - Rollback en caso de error
REM - Funciona tipo "npm install -g"
REM ============================================================================

setlocal EnableDelayedExpansion

echo ╔════════════════════════════════════════════════════════════╗
echo ║         SELFIDEX v3.0 - Instalador para Windows           ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM Verificar permisos de administrador
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Se requieren permisos de administrador.
    echo [INFO] Click derecho en este script -^> "Ejecutar como administrador"
    pause
    exit /b 1
)

echo [OK] Permisos de administrador verificados.
echo.

REM ============================================================================
# CONFIGURACIÓN
REM ============================================================================
set "SELFIDX_VERSION=3.0.0"
set "INSTALL_DIR=%LOCALAPPDATA%\selfidx"
set "BACKUP_DIR=%INSTALL_DIR\backups"
set "SELFIDX_EXE=%INSTALL_DIR\selfidx.exe"

REM ============================================================================
# CREAR DIRECTORIOS
REM ============================================================================
echo [1/6] Creando directorios de instalación...

if not exist "%INSTALL_DIR%" (
    mkdir "%INSTALL_DIR%"
    echo      - Creado: %INSTALL_DIR%
)

if not exist "%BACKUP_DIR%" (
    mkdir "%BACKUP_DIR%"
    echo      - Creado: %BACKUP_DIR%
)

echo.

REM ============================================================================
# BACKUP DEL PATH ACTUAL
REM ============================================================================
echo [2/6] Creando backup del PATH actual...

set "BACKUP_FILE=%BACKUP_DIR\path_backup_%DATE:~-4,4%%DATE:~-7,2%%DATE:~-10,2%_%TIME:~0,2%%TIME:~3,2%.reg"
set "BACKUP_FILE=!BACKUP_FILE: =0!"

REM Exportar PATH actual del registro
reg export "HKCU\Environment" "%BACKUP_FILE%" /y >nul 2>&1
if !errorlevel! equ 0 (
    echo      - Backup creado: !BACKUP_FILE!
) else (
    echo      - [WARNING] No se pudo exportar el PATH (puede ser la primera vez)
)

REM Guardar PATH en archivo de texto también
set "PATH_BACKUP_TXT=%BACKUP_DIR\path_backup.txt"
echo %PATH% > "!PATH_BACKUP_TXT!"
echo      - Backup de texto: !PATH_BACKUP_TXT!
echo.

REM ============================================================================
# COPIAR EJECUTABLE
REM ============================================================================
echo [3/6] Copiando ejecutable de SELFIDEX...

REM Buscar el ejecutable en diferentes ubicaciones
set "SOURCE_EXE="

if exist "target\release\selfidx.exe" (
    set "SOURCE_EXE=target\release\selfidx.exe"
) else if exist "..\target\release\selfidx.exe" (
    set "SOURCE_EXE=..\target\release\selfidx.exe"
) else if exist "selfidx.exe" (
    set "SOURCE_EXE=selfidx.exe"
) else if exist "%CD%\selfidx.exe" (
    set "SOURCE_EXE=%CD%\selfidx.exe"
)

if "!SOURCE_EXE!"=="" (
    echo [ERROR] No se encontró selfidx.exe
    echo [INFO] Primero ejecuta: cargo build --release
    pause
    exit /b 1
)

echo      - Origen: !SOURCE_EXE!
copy /Y "!SOURCE_EXE!" "%SELFIDX_EXE%" >nul
if !errorlevel! equ 0 (
    echo      - [OK] Ejecutable copiado a: %SELFIDX_EXE%
) else (
    echo [ERROR] Error al copiar el ejecutable
    pause
    exit /b 1
)
echo.

REM ============================================================================
# AGREGAR AL PATH DE WINDOWS (SEGURO)
REM ============================================================================
echo [4/6] Agregando SELFIDEX al PATH de Windows...

REM Método SEGURO: Usar PowerShell para modificar el PATH
REM Esto NO elimina el PATH existente, solo agrega al final

powershell -Command " ^
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User'); ^
    $selfidxPath = '%INSTALL_DIR%'; ^
    if ($currentPath -notlike "*$selfidxPath*") { ^
        $newPath = $currentPath + ';' + $selfidxPath; ^
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User'); ^
        Write-Host 'PATH actualizado correctamente'; ^
    } else { ^
        Write-Host 'SELFIDEX ya está en el PATH'; ^
    } ^
"

if !errorlevel! equ 0 (
    echo      - [OK] SELFIDEX agregado al PATH del usuario
) else (
    echo [ERROR] Error al modificar el PATH
    echo [INFO] Puedes agregarlo manualmente: %INSTALL_DIR%
    goto :ROLLBACK_PROMPT
)

echo.

REM ============================================================================
# VERIFICAR INSTALACIÓN
REM ============================================================================
echo [5/6] Verificando instalación...

REM Actualizar PATH en esta sesión
set "PATH=%PATH%;%INSTALL_DIR%"

REM Verificar que el ejecutable existe
if exist "%SELFIDX_EXE%" (
    echo      - [OK] Ejecutable verificado: %SELFIDX_EXE%
    
    REM Mostrar versión
    "%SELFIDX_EXE%" --version >nul 2>&1
    if !errorlevel! equ 0 (
        for /f "tokens=*" %%i in ('"%SELFIDX_EXE%" --version 2^>^&1') do set "VERSION_OUTPUT=%%i"
        echo      - [OK] Versión: !VERSION_OUTPUT!
    ) else (
        echo      - [INFO] Ejecutable listo (sin versión disponible)
    )
) else (
    echo [ERROR] El ejecutable no existe después de la instalación
    goto :ROLLBACK_PROMPT
)

echo.

REM ============================================================================
# RESUMEN
REM ============================================================================
echo [6/6] Instalación completada!
echo.
echo ╔════════════════════════════════════════════════════════════╗
echo ║              ✓ INSTALACIÓN EXITOSA                         ║
echo ╚════════════════════════════════════════════════════════════╝
echo.
echo   SELFIDEX v%SELFIDX_VERSION% se instaló correctamente.
echo.
echo   Ubicación: %INSTALL_DIR%
echo.
echo   Para usar:
echo     1. Cierra esta ventana
echo     2. Abre una NUEVA terminal (PowerShell, CMD, etc.)
echo     3. Ejecuta: selfidx --help
echo.
echo   Comandos útiles:
echo     selfidx --tui         # Iniciar interfaz gráfica
echo     selfidx --chat        # Chat interactivo
echo     selfidx provider list # Ver proveedores LLM
echo     selfidx --help        # Ver todos los comandos
echo.
echo   Backup del PATH:
echo     %BACKUP_DIR%
echo.
echo   Para desinstalar:
echo     Ejecuta: %INSTALL_DIR%\uninstall.bat
echo.
pause

exit /b 0

REM ============================================================================
# ROLLBACK EN CASO DE ERROR
REM ============================================================================
:ROLLBACK_PROMPT
echo.
echo [!] Ocurrió un error durante la instalación.
echo.
set /p RESTORE="¿Deseas restaurar el PATH desde el backup? (S/N): "
if /i "!RESTORE!"=="S" (
    echo Restaurando PATH desde backup...
    
    if exist "!BACKUP_FILE!" (
        reg import "!BACKUP_FILE!" >nul 2>&1
        echo [OK] PATH restaurado desde: !BACKUP_FILE!
    ) else (
        echo [WARNING] No se encontró el archivo de backup
    )
    
    echo.
    echo También puedes restaurar manualmente desde:
    echo   !PATH_BACKUP_TXT!
)

echo.
echo Para soporte, revisa: https://github.com/zerumen82/SELFIDX
pause
exit /b 1
