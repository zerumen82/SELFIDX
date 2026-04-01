@echo off
REM ============================================================================
REM SELFIDEX v3.0 - Desinstalador Seguro para Windows
REM ============================================================================
REM Características:
REM - Elimina SELFIDEX del PATH
REM - Elimina archivos de instalación
REM - Mantiene backups del PATH
REM - Opcional: eliminar logs y configuración
REM ============================================================================

setlocal EnableDelayedExpansion

echo ╔════════════════════════════════════════════════════════════╗
echo ║       SELFIDEX v3.0 - Desinstalador para Windows          ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM Verificar permisos de administrador
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Se requieren permisos de administrador.
    echo [INFO] Click derecho -^> "Ejecutar como administrador"
    pause
    exit /b 1
)

set "INSTALL_DIR=%LOCALAPPDATA%\selfidx"
set "SELFIDX_EXE=%INSTALL_DIR%\selfidx.exe"

REM ============================================================================
# CONFIRMACIÓN
REM ============================================================================
echo Esto desinstalará SELFIDEX de tu sistema.
echo.
echo Archivos que se eliminarán:
echo   - %SELFIDX_EXE%
echo   - Scripts de instalación
echo.
echo El PATH de Windows se restaurará al estado anterior.
echo.
echo Los siguientes archivos se CONSERVARÁN:
echo   - Backups del PATH en: %INSTALL_DIR%\backups\
echo   - Logs en: %INSTALL_DIR%\logs\ (si existen)
echo   - Configuración en: %APPDATA%\selfidx\ (si existe)
echo.

set /p CONFIRM="¿Continuar con la desinstalación? (S/N): "
if /i not "!CONFIRM!"=="S" (
    echo.
    echo Desinstalación cancelada.
    pause
    exit /b 0
)

echo.

REM ============================================================================
# ELIMINAR DEL PATH
REM ============================================================================
echo [1/4] Eliminando SELFIDEX del PATH de Windows...

powershell -Command " ^
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User'); ^
    $selfidxPath = '%INSTALL_DIR%'; ^
    $newPath = ($currentPath.Split(';') | Where-Object { $_ -ne $selfidxPath }) -join ';'; ^
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User'); ^
    Write-Host 'SELFIDEX eliminado del PATH'; ^
"

if !errorlevel! equ 0 (
    echo      - [OK] SELFIDEX eliminado del PATH
) else (
    echo      - [WARNING] Error al modificar el PATH (puede requerir reinicio)
)

echo.

REM ============================================================================
# DETENER PROCESOS SELFIDEX
REM ============================================================================
echo [2/4] Deteniendo procesos de SELFIDEX...

taskkill /F /IM selfidx.exe >nul 2>&1
if !errorlevel! equ 0 (
    echo      - [OK] Procesos detenidos
) else (
    echo      - [INFO] No hay procesos de SELFIDEX en ejecución
)

echo.

REM ============================================================================
# ELIMINAR ARCHIVOS
REM ============================================================================
echo [3/4] Eliminando archivos de instalación...

if exist "%SELFIDX_EXE%" (
    del /F /Q "%SELFIDX_EXE%"
    echo      - Eliminado: %SELFIDX_EXE%
)

if exist "%INSTALL_DIR%\install.bat" (
    del /F /Q "%INSTALL_DIR%\install.bat"
    echo      - Eliminado: %INSTALL_DIR%\install.bat
)

if exist "%INSTALL_DIR%\uninstall.bat" (
    echo      - [INFO] Este archivo se eliminará al final
)

echo.

REM ============================================================================
# PRESERVAR BACKUPS Y LOGS
REM ============================================================================
echo [4/4] Preservando backups y configuración...

if exist "%INSTALL_DIR%\backups" (
    echo      - Backups conservados en: %INSTALL_DIR%\backups\
    echo        (Puedes eliminarlos manualmente si lo deseas)
)

if exist "%APPDATA%\selfidx" (
    echo      - Configuración conservada en: %APPDATA%\selfidx\
    echo        (Para eliminar: rmdir /s %APPDATA%\selfidx)
)

echo.

REM ============================================================================
# ELIMINAR DIRECTORIO PRINCIPAL (si está vacío)
REM ============================================================================
REM Solo eliminar si no hay backups o logs importantes
dir "%INSTALL_DIR%" /b >nul 2>&1
if !errorlevel! equ 0 (
    REM Hay archivos, no eliminar directorio
    echo [INFO] Directorio %INSTALL_DIR% no eliminado (contiene backups/logs)
) else (
    REM Directorio vacío, eliminar
    rmdir "%INSTALL_DIR%" 2>nul
    echo      - Directorio %INSTALL_DIR% eliminado
)

echo.

REM ============================================================================
# RESUMEN
REM ============================================================================
echo ╔════════════════════════════════════════════════════════════╗
echo ║           ✓ DESINSTALACIÓN COMPLETADA                      ║
echo ╚════════════════════════════════════════════════════════════╝
echo.
echo   SELFIDEX ha sido desinstalado de tu sistema.
echo.
echo   Notas:
echo     - El PATH de Windows fue actualizado
echo     - Es posible que necesites reiniciar la terminal
echo     - Los backups se conservan en: %INSTALL_DIR%\backups\
echo.
echo   Para reinstalar:
echo     Ejecuta: scripts\install.bat desde el repositorio
echo.

REM Auto-eliminar este script de desinstalación
timeout /t 2 /nobreak >nul
del /F /Q "%~f0" >nul 2>&1

pause
exit /b 0
