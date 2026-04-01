@echo off
setlocal enabledelayedexpansion

echo ==============================================
echo   INSTALADOR SELFIDEX v3.0
echo ==============================================
echo.

:: Use user's local directory instead of Program Files (no admin needed)
set "INSTALL_DIR=%LOCALAPPDATA%\selfidx"
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

echo [3/4] Agregando al PATH del usuario...

:: Check if already in PATH
echo %PATH% | find /I "%BIN_DIR%" >nul
if %errorlevel% equ 0 (
    echo [INFO] Ya esta en el PATH.
) else (
    :: Get current user PATH from registry to avoid duplication
    for /f "tokens=2*" %%A in ('reg query "HKCU\Environment" /v PATH 2^>nul') do set "USER_PATH=%%B"
    
    :: Check if user PATH already contains the install directory
    echo %USER_PATH% | find /I "%BIN_DIR%" >nul
    if %errorlevel% equ 0 (
        echo [INFO] Ya esta en el PATH del usuario.
    ) else (
        :: Use setx to modify user PATH (not system PATH)
        setx PATH "%USER_PATH%;%BIN_DIR%" >nul 2>&1
        if %errorlevel% neq 0 (
            echo [ADVERTENCIA] No se pudo modificar el PATH automaticamente.
            echo [INFO] Puedes agregar manualmente: %BIN_DIR%
            echo [INFO] O ejecuta: setx PATH "%%PATH%%;%BIN_DIR%"
        ) else (
            echo [OK] Agregado al PATH del usuario.
        )
    )
)

echo [4/4] Creando acceso directo en el menu Inicio...
powershell -Command "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%USERPROFILE%\Start Menu\Programs\SELFIDEX.lnk'); $Shortcut.TargetPath = '%BIN_DIR%\%EXE_NAME%'; $Shortcut.WorkingDirectory = '%INSTALL_DIR%'; $Shortcut.Description = 'SELFIDEX v3.0 - Terminal con IA'; $Shortcut.Save()"

echo.
echo ==============================================
echo   INSTALACION COMPLETADA
echo ==============================================
echo.
echo Ubicacion: %INSTALL_DIR%
echo.
echo Para usar, abre una NUEVA terminal y ejecuta:
echo   selfidx --help
echo.
echo NOTA: Si 'selfidx' no funciona, reinicia tu terminal.
echo.

pause
