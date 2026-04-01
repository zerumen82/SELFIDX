@echo off
setlocal enabledelayedexpansion

echo ==============================================
echo   INSTALADOR SELFIDEX v3.0
echo ==============================================
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\selfidx"
set "EXE_SOURCE=%~dp0..\target\release\selfidx.exe"

:: Check if exe exists
if not exist "%EXE_SOURCE%" (
    echo [ERROR] No se encontró selfidx.exe
    echo [INFO] Ejecuta primero: cargo build --release
    pause
    exit /b 1
)

echo [1/3] Creando directorio de instalacion...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo [2/3] Copiando ejecutable...
copy /Y "%EXE_SOURCE%" "%INSTALL_DIR%\selfidx.exe" >nul
if %errorlevel% neq 0 (
    echo [ERROR] No se pudo copiar el ejecutable.
    pause
    exit /b 1
)

echo [3/3] Agregando al PATH del usuario...

:: Check if PATH already contains the install directory
echo %PATH% | find /I "%INSTALL_DIR%" >nul
if %errorlevel% equ 0 (
    echo [INFO] Ya está en el PATH.
) else (
    :: Get current user PATH from registry to avoid duplication
    for /f "tokens=2*" %%A in ('reg query "HKCU\Environment" /v PATH 2^>nul') do set "USER_PATH=%%B"
    
    :: Check if user PATH already contains the install directory
    echo %USER_PATH% | find /I "%INSTALL_DIR%" >nul
    if %errorlevel% equ 0 (
        echo [INFO] Ya está en el PATH del usuario.
    ) else (
        :: Add to user PATH (not system PATH)
        setx PATH "%USER_PATH%;%INSTALL_DIR%" >nul 2>&1
        if %errorlevel% neq 0 (
            echo [ADVERTENCIA] No se pudo modificar el PATH automaticamente.
            echo [INFO] Puedes agregar manualmente: %INSTALL_DIR%
        ) else (
            echo [OK] Agregado al PATH del usuario.
        )
    )
)

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
