# MindVault Flotante

Aplicación de escritorio liviana construida con Tauri 2.x que vive siempre visible sobre todas las ventanas del sistema. Permite capturar texto rápidamente y enviarlo a MindVault Server — ya sea vía subprocess de `claude` o por REST como fallback.

---

## Requisitos

### Windows
- Windows 10 o superior (64-bit)
- WebView2 Runtime (preinstalado en Windows 11; en Windows 10 se descarga automáticamente con el instalador)
- MindVault Server corriendo (ver sección al final)

### macOS
- macOS 10.15 (Catalina) o superior
- MindVault Server corriendo

### Linux
- Distribución moderna con soporte GTK 3 y WebKitGTK
- Dependencias del sistema: `libwebkit2gtk-4.1`, `libappindicator3`, `librsvg2`
- MindVault Server corriendo

---

## Setup de desarrollo

### Entorno Windows + WSL2

Este proyecto se edita en WSL2 y se compila/ejecuta desde Windows nativo.

**Paso 1: Instalar herramientas en Windows (una vez)**

Abrir PowerShell como administrador y ejecutar:

```powershell
# Instalar Rust en Windows
winget install Rustlang.Rustup

# Instalar Visual Studio Build Tools (requerido por el compilador Rust en Windows)
winget install Microsoft.VisualStudio.2022.BuildTools

# Instalar Node.js LTS
winget install OpenJS.NodeJS.LTS
```

Luego cerrar y reabrir PowerShell para que los cambios de PATH tengan efecto.

**Paso 2: Instalar Tauri CLI**

```powershell
cargo install tauri-cli
```

**Paso 3: Acceder al proyecto desde Windows**

El proyecto vive en el filesystem de WSL2. Desde Windows es accesible como:

```
\\wsl$\Ubuntu\home\marioyahuar\saasProjects\MindVaultFlotante
```

Para abrir PowerShell directamente en esa carpeta:

1. Abrir el Explorador de Windows y navegar a `\\wsl$\Ubuntu\home\marioyahuar\saasProjects\MindVaultFlotante`
2. En la barra de direcciones escribir `powershell` y presionar Enter

O desde PowerShell directamente:

```powershell
cd "\\wsl$\Ubuntu\home\marioyahuar\saasProjects\MindVaultFlotante"
```

**Paso 4: Instalar dependencias Node**

Desde PowerShell en la carpeta del proyecto:

```powershell
npm install
```

### Ejecutar en desarrollo

Desde PowerShell en la carpeta del proyecto:

```powershell
cargo tauri dev
```

Esto levanta el servidor Vite para el frontend (puerto 1420) y la ventana Tauri con live reload.

### Build de distribución

Para generar el instalador (.exe + .msi en Windows):

```powershell
cargo tauri build
```

Los artefactos quedan en `src-tauri/target/release/bundle/`.

---

## Configuración

La URL del MindVault Server se lee desde un archivo `config.json`. La ubicación varía por plataforma:

| Plataforma | Ruta |
|------------|------|
| Windows | `C:\Users\{usuario}\AppData\Roaming\mindvault-flotante\config.json` |
| macOS | `~/Library/Application Support/mindvault-flotante/config.json` |
| Linux | `~/.local/share/mindvault-flotante/config.json` |

Si el archivo no existe al arrancar, se crea automáticamente con la URL por defecto:

```json
{
  "server_url": "http://localhost:3000"
}
```

Para apuntar a un servidor remoto o en otro puerto, editar el archivo manualmente y reiniciar la aplicación.

---

## MindVault Server

El Flotante requiere que MindVault Server esté corriendo para el flujo de fallback REST. Si el servidor no está activo, el indicador "Server" aparecerá en rojo.

Para arrancar el servidor (en el directorio de MindVault Server):

```bash
npm run dev
# o en producción:
npm start
```

El Flotante verificará automáticamente el estado del servidor cada 10 segundos y actualizará el indicador sin necesidad de reiniciar.

---

## Hotkey global

Por defecto: `Ctrl+Shift+M` en Windows y Linux, `Cmd+Shift+M` en macOS.

Trae la ventana al frente desde cualquier aplicación, incluso si está minimizada al tray.

---

## Uso

1. Escribir o pegar texto en el área de entrada
2. Presionar el botón **Guardar**
3. Si `claude` está disponible en el PATH: el texto se guarda directamente (indicador verde "Guardado")
4. Si `claude` no está disponible o falla: se envía al servidor como captura pendiente (indicador amarillo "Guardado como pendiente")
5. Cerrar la ventana con la X la minimiza al tray — para cerrar completamente, usar el menú del ícono en la barra del sistema y seleccionar **Salir**
