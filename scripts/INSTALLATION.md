# Guía de Instalación - SELFIDEX v3.0

## 📦 Métodos de Instalación

SELFIDEX ofrece **4 métodos de instalación** seguros que **NO eliminan el PATH de Windows**.

---

## ⚡ Método 1: Instalación Rápida (Recomendado)

**Tipo**: `npm install -g selfidx`

```bash
# Desde el repositorio
cd D:\PROJECTS\SELFIDEX_V3\scripts
.\install-simple.bat
```

**Qué hace:**
1. ✅ Clona/actualiza el repositorio
2. ✅ Compila en modo release
3. ✅ Copia el ejecutable a `%LOCALAPPDATA%\selfidx`
4. ✅ Agrega al PATH de forma segura
5. ✅ Verifica la instalación

**Tiempo**: ~2-5 minutos (depende de tu CPU)

**Ventajas:**
- ✅ Un solo comando
- ✅ No requiere administrador
- ✅ Fácil de actualizar (ejecutar de nuevo)

---

## 🛠️ Método 2: Instalación Manual

**Para usuarios que ya compil el proyecto:**

```bash
# Como administrador
cd D:\PROJECTS\SELFIDEX_V3\scripts
.\install.bat
```

**Qué hace:**
1. ✅ Crea backup del PATH actual
2. ✅ Copia `selfidx.exe` a `%LOCALAPPDATA%\selfidx`
3. ✅ Agrega al PATH con PowerShell (seguro)
4. ✅ Crea archivo de rollback

**Requiere**: Permisos de administrador

**Backup del PATH:**
```
%LOCALAPPDATA%\selfidx\backups\path_backup_YYYYMMDD_HHMMSS.reg
```

---

## 🖥️ Método 3: Instalador Gráfico (.exe)

**Requiere**: Inno Setup 6 (opcional)

### Generar el instalador:

```bash
cd D:\PROJECTS\SELFIDEX_V3\scripts
.\build-all.bat
```

**O manualmente:**
```bash
# 1. Compilar
cargo build --release

# 2. Generar instalador
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer.iss
```

**El instalador:**
- ✅ Muestra wizard gráfico
- ✅ Crea backup del PATH antes de instalar
- ✅ Permite seleccionar opciones
- ✅ Crea accesos directos
- ✅ Agrega al PATH automáticamente

**Características de seguridad:**
- ✅ Backup automático del PATH
- ✅ Verifica si ya está instalado
- ✅ Rollback en caso de error
- ✅ No requiere administrador

---

## 📦 Método 4: Instalador MSI

**Requiere**: cargo-bundle

### Generar MSI:

```bash
# Instalar cargo-bundle (una vez)
cargo install cargo-bundle

# Generar MSI
cd D:\PROJECTS\SELFIDEX_V3
cargo bundle --release --format msi
```

**El MSI se genera en:**
```
target\bundle\msi\SELFIDEX-3.0.0.msi
```

**Ventajas:**
- ✅ Estándar de Windows
- ✅ Se puede desplegar con Group Policy
- ✅ No requiere scripts

---

## 🔒 Seguridad del PATH

### ¿Cómo protegemos tu PATH?

**1. Backup automático:**
```batch
reg export "HKCU\Environment" "path_backup_YYYYMMDD.reg" /y
```

**2. Modificación segura (PowerShell):**
```powershell
$currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$selfidxPath = 'C:\Users\Tu\AppData\Local\selfidx'

# VERIFICAR si ya está en el PATH
if ($currentPath -notlike "*$selfidxPath*") {
    # AGREGAR al final (NO reemplazar)
    $newPath = $currentPath + ';' + $selfidxPath
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
}
```

**3. Rollback en caso de error:**
```batch
# Si algo sale mal, restaura desde backup
reg import "path_backup_YYYYMMDD.reg"
```

---

## ✅ Verificar Instalación

```bash
# Abrir NUEVA terminal y ejecutar:
selfidx --version

# Debería mostrar:
selfidx 3.0.0
```

**Si no funciona:**
1. Cierra TODAS las terminales
2. Abre una NUEVA terminal
3. Ejecuta: `selfidx --version`

**Si aún no funciona:**
```bash
# Verificar PATH manualmente
echo %PATH% | findstr selfidx

# Debería mostrar algo como:
# ...;C:\Users\Tu\AppData\Local\selfidx;...
```

---

## 🗑️ Desinstalación

### Método 1: Script de desinstalación

```bash
# Como administrador
cd D:\PROJECTS\SELFIDEX_V3\scripts
.\uninstall.bat
```

**Qué hace:**
1. ✅ Elimina SELFIDEX del PATH
2. ✅ Detiene procesos de SELFIDEX
3. ✅ Elimina archivos de instalación
4. ✅ **Conserva** backups y configuración

### Método 2: Manual

```bash
# 1. Eliminar del PATH (PowerShell como admin)
$currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$newPath = ($currentPath.Split(';') | Where-Object { $_ -notlike '*selfidx*' }) -join ';'
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')

# 2. Eliminar directorio
rmdir /s %LOCALAPPDATA%\selfidx
```

### Método 3: Windows Settings

```
Settings → Apps → Installed Apps → SELFIDEX → Uninstall
```

(Si se instaló con MSI)

---

## 🔄 Actualización

```bash
# Ejecutar el script de instalación nuevamente
cd D:\PROJECTS\SELFIDEX_V3\scripts
.\install-simple.bat
```

El script:
1. Hace `git pull` para obtener la última versión
2. Compila la nueva versión
3. Reemplaza el ejecutable anterior
4. **Mantiene** tu configuración y backups

---

## 📁 Archivos de Instalación

```
scripts/
├── install.bat              # Instalación manual (admin)
├── install-simple.bat       # Instalación rápida (npm-style)
├── uninstall.bat            # Desinstalación segura
├── build-all.bat            # Genera todos los instaladores
├── installer.iss            # Script Inno Setup
└── INSTALLATION.md          # Este archivo
```

**Después de build:**
```
dist/
├── selfidx.exe              # Ejecutable standalone
├── install.bat              # Copia de install scripts
├── install-simple.bat
├── uninstall.bat
├── SELFIDEX-Setup-3.0.0.exe # Instalador Inno Setup
└── SELFIDEX-3.0.0.msi       # Instalador MSI (opcional)
```

---

## 🆘 Solución de Problemas

### "selfidx no se reconoce como comando"

**Solución 1**: Reiniciar terminal
```
Cierra TODAS las terminales y abre una nueva
```

**Solución 2**: Verificar PATH
```bash
echo %PATH% | findstr selfidx
```

**Solución 3**: Agregar manualmente
```bash
# PowerShell (como usuario normal)
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$newPath = $userPath + ';%LOCALAPPDATA%\selfidx'
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
```

### "Acceso denegado"

**Solución**: Ejecutar como administrador
```
Click derecho en el script → "Ejecutar como administrador"
```

### "El PATH se eliminó/cortó"

**Esto NO debería pasar con nuestros scripts**, pero si ocurre:

**Restaurar desde backup:**
```bash
# Los backups están en:
%LOCALAPPDATA%\selfidx\backups\

# Restaurar manualmente:
reg import "%LOCALAPPDATA%\selfidx\backups\path_backup_YYYYMMDD_HHMMSS.reg"
```

**O desde archivo de texto:**
```bash
# Copiar el contenido de:
%LOCALAPPDATA%\selfidx\backups\path_backup.txt

# Y pegar en:
# System Properties → Environment Variables → Path → Edit
```

---

## 📊 Comparación de Métodos

| Método | Admin | Velocidad | Facilidad | Recomendado |
|--------|-------|-----------|-----------|-------------|
| **install-simple.bat** | ❌ No | Media | ⭐⭐⭐⭐⭐ | ✅ Sí |
| **install.bat** | ✅ Sí | Rápida | ⭐⭐⭐⭐ | ✅ Sí |
| **Setup.exe** | ❌ No | Rápida | ⭐⭐⭐⭐⭐ | ✅ Sí |
| **MSI** | ❌ No | Rápida | ⭐⭐⭐⭐ | ✅ Sí |

---

## 🎯 Recomendaciones

### Para Desarrollo
```bash
# Usar install-simple.bat
.\install-simple.bat

# Para actualizar en el futuro:
# Ejecutar el mismo comando nuevamente
```

### Para Producción/Distribución
```bash
# Generar instalador .exe o .msi
.\build-all.bat

# Distribuir: SELFIDEX-Setup-3.0.0.exe
```

### Para Enterprise
```bash
# Usar MSI con Group Policy
cargo bundle --release --format msi

# Deploy con: msiexec /i SELFIDEX-3.0.0.msi /quiet
```

---

**Última actualización**: 2 de abril de 2026  
**Versión**: 3.0.0  
**Estado**: ✅ Probado en Windows 10/11
