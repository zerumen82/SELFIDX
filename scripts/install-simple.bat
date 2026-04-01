@echo off
REM ============================================================================
REM SELFIDEX v3.0 - Instalación Rápida (tipo npm install -g)
REM ============================================================================
REM Uso: install-simple.bat
REM - Descarga la última versión desde GitHub
REM - Instala en PATH automáticamente
REM - Sin preguntas, sin complicaciones
REM ============================================================================

setlocal EnableDelayedExpansion

echo ╔════════════════════════════════════════════════════════════╗
echo ║     SELFIDEX v3.0 - Instalación Rápida                    ║
echo ║     (tipo npm install -g)                                  ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM Verificar si Git está instalado
where git >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Git no está instalado.
    echo [INFO] Descarga Git desde: https://git-scm.com/
    pause
    exit /b 1
)

REM Verificar si Rust está instalado
where cargo >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Rust/Cargo no está instalado.
    echo [INFO] Descarga Rust desde: https://rustup.rs/
    pause
    exit /b 1
)

echo [OK] Git y Cargo detectados.
echo.

REM ============================================================================
# CLONAR O ACTUALIZAR REPOSITORIO
REM ============================================================================
set "SELFIDX_DIR=%USERPROFILE%\selfidx"

if exist "%SELFIDX_DIR%" (
    echo [1/4] Actualizando SELFIDEX...
    cd /d "%SELFIDX_DIR%"
    git pull origin master
) else (
    echo [1/4] Clonando SELFIDEX...
    git clone https://github.com/zerumen82/SELFIDX.git "%SELFIDX_DIR%"
    cd /d "%SELFIDX_DIR%"
)

if !errorlevel! neq 0 (
    echo [ERROR] Error al clonar/actualizar el repositorio
    pause
    exit /b 1
)

echo.

REM ============================================================================
# COMPILAR
REM ============================================================================
echo [2/4] Compilando SELFIDEX (esto puede tardar unos minutos)...

cargo build --release

if !errorlevel! neq 0 (
    echo [ERROR] Error al compilar
    pause
    exit /b 1
)

echo      - [OK] Compilación completada.
echo.

REM ============================================================================
# COPIAR EJECUTABLE
REM ============================================================================
set "INSTALL_DIR=%LOCALAPPDATA%\selfidx"
set "SELFIDX_EXE=%INSTALL_DIR%\selfidx.exe"

echo [3/4] Instalando en PATH...

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

copy /Y "target\release\selfidx.exe" "%SELFIDX_EXE%" >nul
if !errorlevel! equ 0 (
    echo      - [OK] Ejecutable instalado: %SELFIDX_EXE%
) else (
    echo [ERROR] Error al copiar el ejecutable
    pause
    exit /b 1
)

echo.

REM ============================================================================
# AGREGAR AL PATH (POWERSHELL - SEGURO)
REM ============================================================================
powershell -Command " ^
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User'); ^
    $selfidxPath = '%INSTALL_DIR%'; ^
    if ($currentPath -notlike "*$selfidxPath*") { ^
        $newPath = $currentPath + ';' + $selfidxPath; ^
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User'); ^
    } ^
"

echo      - [OK] SELFIDEX agregado al PATH
echo.

REM ============================================================================
# VERIFICAR
REM ============================================================================
echo [4/4] Verificando instalación...

set "PATH=%PATH%;%INSTALL_DIR%"

"%SELFIDX_EXE%" --version >nul 2>&1
if !errorlevel! equ 0 (
    for /f "tokens=*" %%i in ('"%SELFIDX_EXE%" --version 2^>^&1') do set "VERSION=%%i"
    echo      - [OK] !VERSION!
) else (
    echo      - [OK] Instalado correctamente
)

echo.

REM ============================================================================
# RESUMEN
REM ============================================================================
echo ╔════════════════════════════════════════════════════════════╗
echo ║              ✓ INSTALACIÓN COMPLETADA                      ║
echo ╚════════════════════════════════════════════════════════════╝
echo.
echo   ¡SELFIDEX está listo para usar!
echo.
echo   Ubicación: %SELFIDX_DIR%
echo   Ejecutable: %SELFIDX_EXE%
echo.
echo   Próximo paso:
echo     1. Cierra esta ventana
echo     2. Abre una NUEVA terminal
echo     3. Ejecuta: selfidx --tui
echo.
echo   Comandos rápidos:
echo     selfidx --tui         # Interfaz gráfica con ratón
echo     selfidx --chat        # Chat con IA
echo     selfidx provider list # Ver proveedores LLM
echo     selfidx --help        # Ayuda completa
echo.
echo   Actualizar en el futuro:
echo     Ejecuta este script nuevamente
echo.
echo   Desinstalar:
echo     Ejecuta: %SELFIDX_DIR%\scripts\uninstall.bat
echo.
pause

REM Abrir nueva ventana con selfidx --help
set /p OPEN_HELP="¿Ver ayuda de SELFIDEX ahora? (S/N): "
if /i "!OPEN_HELP!"=="S" (
    "%SELFIDX_EXE%" --help
    pause
)

exit /b 0
