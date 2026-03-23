@echo off
setlocal enabledelayedexpansion

echo ==============================================
echo   BUILD SELFIDEX v3.0 + CARGO-BUNDLE
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

:: Check if cargo-bundle is installed
where cargo-bundle >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] cargo-bundle no está instalado. Instalando...
    cargo install cargo-bundle
    if %errorlevel% neq 0 (
        echo [ERROR] Error al instalar cargo-bundle.
        pause
        exit /b 1
    )
    echo [OK] cargo-bundle instalado correctamente.
    echo.
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

echo [3/3] Generando instalador con cargo-bundle...
cd ..
cargo bundle --release --format msi
if %errorlevel% neq 0 (
    echo [ADVERTENCIA] Error al generar instalador MSI.
    echo [INFO] Intentando generar solo el ejecutable...
    echo.
    echo [OK] Ejecutable disponible en: dist\selfidx.exe
    echo [INFO] Puedes usar el ejecutable directamente o instalar manualmente.
) else (
    echo [OK] Instalador generado en target\bundle\msi\
    echo [INFO] Copiando instalador a dist\...
    copy /Y "target\bundle\msi\*.msi" "dist\" >nul 2>&1
    if %errorlevel% equ 0 (
        echo [OK] Instalador copiado a dist\
    )
)

echo.
echo ==============================================
echo   BUILD COMPLETADO
echo ==============================================
echo.
echo Archivos generados:
echo   - Ejecutable: dist\selfidx.exe
if exist "dist\*.msi" (
    echo   - Instalador: dist\*.msi
)
echo.
echo Para instalar:
echo   1. Ejecuta dist\selfidx.exe directamente, O
if exist "dist\*.msi" (
    echo   2. Ejecuta el instalador MSI en dist\
)
echo.
echo Para usar cargo-bundle manualmente:
echo   cargo bundle --release --format msi
echo.
pause
