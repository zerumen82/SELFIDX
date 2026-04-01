@echo off
REM ============================================================================
REM SELFIDEX v3.0 - Build Script Completo
REM ============================================================================
REM Genera:
REM - Ejecutable standalone (selfidx.exe)
REM - Instalador tipo npm (install-simple.bat)
REM - Instalador MSI (con cargo-bundle)
REM - Instalador Inno Setup (.exe)
REM ============================================================================

setlocal EnableDelayedExpansion

echo ╔════════════════════════════════════════════════════════════╗
echo ║         SELFIDEX v3.0 - Build Script                      ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM ============================================================================
# VERIFICAR DEPENDENCIAS
REM ============================================================================
echo [INFO] Verificando dependencias...

REM Verificar Rust
where cargo >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Rust no está instalado.
    echo [INFO] https://rustup.rs/
    pause
    exit /b 1
)
echo      - [OK] Rust/Cargo instalado

REM Verificar Inno Setup (opcional)
set "ISCC_PATH="
if exist "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" (
    set "ISCC_PATH=C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
    echo      - [OK] Inno Setup 6 detectado
) else if exist "C:\Program Files\Inno Setup 6\ISCC.exe" (
    set "ISCC_PATH=C:\Program Files\Inno Setup 6\ISCC.exe"
    echo      - [OK] Inno Setup 6 detectado
) else (
    echo      - [INFO] Inno Setup no detectado (opcional)
)

echo.

REM ============================================================================
# COMPILAR
REM ============================================================================
echo [1/5] Compilando SELFIDEX en modo release...

cargo build --release

if !errorlevel! neq 0 (
    echo [ERROR] Error al compilar
    pause
    exit /b 1
)

echo      - [OK] Compilación completada
echo.

REM ============================================================================
# CREAR DIRECTORIO DIST
REM ============================================================================
echo [2/5] Preparando directorio de distribución...

if not exist "..\dist" mkdir "..\dist"

REM Limpiar dist anterior
del /F /Q "..\dist\*.*" >nul 2>&1

echo      - [OK] Directorio dist listo
echo.

REM ============================================================================
# COPIAR EJECUTABLE
REM ============================================================================
echo [3/5] Copiando ejecutable y scripts...

copy /Y "..\target\release\selfidx.exe" "..\dist\selfidx.exe" >nul
echo      - [OK] selfidx.exe copiado

copy /Y "install.bat" "..\dist\install.bat" >nul
echo      - [OK] install.bat copiado

copy /Y "install-simple.bat" "..\dist\install-simple.bat" >nul
echo      - [OK] install-simple.bat copiado

copy /Y "uninstall.bat" "..\dist\uninstall.bat" >nul
echo      - [OK] uninstall.bat copiado

echo.

REM ============================================================================
# GENERAR INSTALADOR INNO SETUP (si está disponible)
REM ============================================================================
echo [4/5] Generando instalador Inno Setup...

if "!ISCC_PATH!"=="" (
    echo      - [SKIP] Inno Setup no detectado
    echo      - [INFO] Para generar el instalador .exe:
    echo      -          1. Instala Inno Setup 6: https://jrsoftware.org/isinfo.php
    echo      -          2. Ejecuta: "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer.iss
) else (
    "!ISCC_PATH!" installer.iss
    
    if !errorlevel! equ 0 (
        echo      - [OK] Instalador generado: ..\dist\SELFIDEX-Setup-3.0.0.exe
    ) else (
        echo      - [ERROR] Error al generar instalador Inno Setup
    )
)

echo.

REM ============================================================================
# GENERAR MSI CON CARGO-BUNDLE (opcional)
REM ============================================================================
echo [5/5] Generando instalador MSI con cargo-bundle...

REM Verificar si cargo-bundle está instalado
where cargo-bundle >nul 2>&1
if %errorLevel! equ 0 (
    echo      - [INFO] cargo-bundle detectado, generando MSI...
    
    cd ..
    cargo bundle --release --format msi
    cd scripts
    
    if !errorlevel! equ 0 (
        echo      - [OK] MSI generado en: ..\target\bundle\msi\
        
        REM Copiar MSI a dist
        if exist "..\target\bundle\msi\*.msi" (
            copy /Y "..\target\bundle\msi\*.msi" "..\dist\" >nul
            echo      - [OK] MSI copiado a dist\
        )
    ) else (
        echo      - [INFO] cargo-bundle no disponible o error al generar MSI
    )
) else (
    echo      - [SKIP] cargo-bundle no instalado
    echo      - [INFO] Para instalar: cargo install cargo-bundle
)

echo.

REM ============================================================================
# RESUMEN
REM ============================================================================
echo ╔════════════════════════════════════════════════════════════╗
echo ║              ✓ BUILD COMPLETADO                           ║
echo ╚════════════════════════════════════════════════════════════╝
echo.
echo   Archivos generados en: ..\dist\
echo.

dir /b "..\dist" | find /c /v "" >nul
for /f %%i in ('dir /b "..\dist"') do (
    echo      - %%i
)

echo.
echo   Métodos de instalación para el usuario:
echo.
echo   1. RÁPIDO (tipo npm):
echo      - Ejecutar: install-simple.bat
echo      - Descarga, compila e instala automáticamente
echo.
echo   2. MANUAL:
echo      - Ejecutar: install.bat (como administrador)
echo      - Copia el ejecutable y agrega al PATH
echo.
echo   3. INSTALADOR .EXE (si se generó):
echo      - Ejecutar: SELFIDEX-Setup-3.0.0.exe
echo      - Instalador gráfico con Inno Setup
echo.
echo   4. INSTALADOR MSI (si se generó):
echo      - Ejecutar: SELFIDEX-3.0.0.msi
echo      - Instalador Windows estándar
echo.
echo   Para desinstalar:
echo      - Ejecutar: uninstall.bat (como administrador)
echo.

pause
exit /b 0
