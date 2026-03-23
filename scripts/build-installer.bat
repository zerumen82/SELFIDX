@echo off
setlocal enabledelayedexpansion

echo ==============================================
echo   BUILD SELFIDEX v3.0 + INSTALADOR
echo ==============================================
echo.

:: Check if Rust is installed
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Rust/Cargo no está instalado.
    echo [INFO] Descarga Rust desde: https://rustup.rs/
    pause
    exit /b 1
)

:: Check if Inno Setup is installed
set "INNO_SETUP="
if exist "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" (
    set "INNO_SETUP=C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
) else if exist "C:\Program Files\Inno Setup 6\ISCC.exe" (
    set "INNO_SETUP=C:\Program Files\Inno Setup 6\ISCC.exe"
) else (
    echo [ADVERTENCIA] Inno Setup no encontrado.
    echo [INFO] Descarga Inno Setup desde: https://jrsoftware.org/isinfo.php
    echo [INFO] O compila manualmente con: cargo build --release
    echo.
    set "INNO_SETUP=NONE"
)

echo [1/3] Compilando proyecto en modo release...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Error al compilar el proyecto.
    pause
    exit /b 1
)

echo [OK] Proyecto compilado correctamente.
echo.

:: Create dist directory
if not exist "..\dist" mkdir "..\dist"

echo [2/3] Copiando ejecutable...
copy /Y "..\target\release\selfidx.exe" "..\dist\selfidx.exe" >nul
if %errorlevel% neq 0 (
    echo [ERROR] Error al copiar ejecutable.
    pause
    exit /b 1
)

echo [OK] Ejecutable copiado a dist\
echo.

if "%INNO_SETUP%"=="NONE" (
    echo [3/3] Instalador no generado (Inno Setup no encontrado).
    echo [INFO] Puedes generar el instalador manualmente con Inno Setup.
    echo [INFO] Archivo script: scripts\installer.iss
) else (
    echo [3/3] Generando instalador con Inno Setup...
    "%INNO_SETUP%" "installer.iss"
    if %errorlevel% neq 0 (
        echo [ERROR] Error al generar instalador.
        pause
        exit /b 1
    )
    echo [OK] Instalador generado en dist\
)

echo.
echo ==============================================
echo   BUILD COMPLETADO
echo ==============================================
echo.
echo Archivos generados:
echo   - Ejecutable: dist\selfidx.exe
if not "%INNO_SETUP%"=="NONE" (
    echo   - Instalador: dist\SELFIDEX-Setup-3.0.0.exe
)
echo.
echo Para instalar:
echo   1. Ejecuta dist\SELFIDEX-Setup-3.0.0.exe
echo   2. O copia dist\selfidx.exe a cualquier ubicación
echo.
pause
