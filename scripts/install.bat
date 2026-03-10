@echo off
setlocal enabledelayedexpansion

echo ==============================================
echo   INSTALADOR SELFIDEX v3.0
echo ==============================================
echo.

:: Check for admin rights
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] Ejecutando como administrador...
    powershell -Command "Start-Process cmd -ArgumentList '/c %~f0' -Verb RunAs"
    exit /b
)

set "INSTALL_DIR=C:\Program Files\SELFIDEX"
set "BIN_DIR=%INSTALL_DIR%"
set "EXE_NAME=selfidx.exe"

echo [1/4] Creando directorio de instalacion...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo [2/4] Copiando archivos...
copy /Y "%~dp0selfidx.exe" "%BIN_DIR%\%EXE_NAME%" >nul
if %errorlevel% neq 0 (
    echo [ERROR] No se pudo copiar el archivo.
    pause
    exit /b 1
)

echo [3/4] Agregando al PATH del sistema...
setx PATH "%PATH%;%BIN_DIR%" >nul 2>&1

echo [4/4] Creando acceso directo en el menu Inicio...
powershell -Command "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%USERPROFILE%\Start Menu\Programs\SELFIDEX.lnk'); $Shortcut.TargetPath = '%BIN_DIR%\%EXE_NAME%'; $Shortcut.WorkingDirectory = '%INSTALL_DIR%'; $Shortcut.Description = 'SELFIDEX v3.0 - Terminal con IA'; $Shortcut.Save()"

echo.
echo ==============================================
echo   INSTALACION COMPLETADA
echo ==============================================
echo.
echo Ubicacion: %INSTALL_DIR%
echo.
echo Para usar, abre una nueva terminal y ejecuta:
echo   selfidx --help
echo.
echo NOTA: Si 'selfidx' no funciona, reinicia tu PC.
echo.

pause
