# Propuesta: MindVault Flotante

**ID:** mindvault-flotante
**Fecha:** 2026-03-24
**Estado:** Borrador — pendiente de aprobación

---

## Intent

El usuario ya tiene MindVault Server funcionando como sistema de memoria personal, pero capturar algo requiere abrir una terminal. El Flotante elimina esa fricción: una ventana mínima always-on-top donde se pega texto y se guarda en MindVault con un click, desde cualquier contexto (Gmail, WhatsApp Web, Teams, etc.).

El valor concreto: pasar de "abrir terminal → escribir comando" a "pegar + click → guardado en segundos", sin interrumpir el flujo de trabajo.

---

## Scope

**Incluido:**
- Aplicación de escritorio Tauri 2.x cross-platform (Windows, macOS, Linux)
- Ventana always-on-top con área de texto + botón de envío
- Flujo principal: subprocess `claude -p "{texto}"` con timeout 30s
- Fallback automático: `POST /captures` si claude falla o hace timeout
- Indicadores de estado visibles: Server activo/inactivo + Claude disponible/no
- Hotkey global para traer la ventana al frente (no auto-envía)
- System tray: minimizar y restaurar sin perder estado
- Configuración de URL del Server via archivo JSON en `app_data_dir`
- Distribución como instalador nativo por plataforma (`.exe`, `.dmg`, `.AppImage`)
- CI/CD con GitHub Actions para builds cross-platform

**Excluido:**
- Autenticación de cualquier tipo
- Pantalla de configuración con UI
- Historial de capturas en la interfaz
- Sincronización multi-dispositivo
- Captura automática sin acción del usuario
- Modificaciones al MindVault Server
- Soporte para múltiples servidores simultáneos

---

## Approach

**Stack:** Tauri 2.x (Rust backend + HTML/TypeScript frontend vanilla).

Tauri fue elegido por: bundle liviano (~10-15MB vs ~150MB de Electron), arranque rápido (~200ms), bajo footprint de RAM (~30-50MB idle), y distribución como ejecutable nativo sin dependencias en Windows y macOS. En Linux requiere WebKit2GTK del sistema, aceptable dado que los usuarios Linux del equipo son desarrolladores.

**Decisiones técnicas confirmadas:**

| Decisión | Elección | Razón |
|----------|----------|-------|
| Frontend | HTML + TypeScript vanilla | UI demasiado simple para justificar un framework |
| HTTP client | `reqwest` async directo | Control total sobre timeouts y errores |
| Config | JSON en `app_data_dir` | Patrón estándar de apps de escritorio |
| Subprocess timeout | `tokio::time::timeout` + `child.kill()` explícito | Evita procesos huérfanos |
| System tray | `tauri-plugin-tray` (oficial) | Plugin mantenido por Tauri team |
| Hotkey global | `tauri-plugin-global-shortcut` (oficial) | Plugin mantenido por Tauri team |

**Workflow de desarrollo (WSL2 + Windows):**

El desarrollo diario ocurre en WSL2: edición de código, control de versiones, linting. El testing funcional se hace desde Windows porque el Flotante es una app con ventana que requiere display nativo. Estrategia:

- `cargo tauri dev` para testing en Windows: ejecutar desde PowerShell/Windows Terminal con Rust instalado en el lado Windows (no en WSL2). El código fuente se edita en WSL2 vía VSCode + extensión Remote WSL.
- Builds de distribución: GitHub Actions buildea `.exe` (Windows runner), `.dmg` (macOS runner) y `.AppImage`/`.deb` (Linux runner). No se hace cross-compilation manual.

---

## Affected Areas

Esta es una aplicación nueva desde cero. No hay código existente que se modifique. Los archivos que se crearán:

```
MindVaultFlotante/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              — punto de entrada Tauri, registro de comandos
│   │   ├── captura.rs           — lógica de subprocess claude + fallback REST
│   │   ├── estado.rs            — health check del Server + verificación claude en PATH
│   │   └── config.rs            — lectura/escritura de configuración JSON
│   ├── Cargo.toml               — dependencias Rust (tauri, reqwest, tokio, serde)
│   ├── tauri.conf.json          — configuración ventana, plugins, permisos
│   └── capabilities/
│       └── default.json         — permisos Tauri 2.x (subprocess, HTTP, tray, shortcut)
├── src/
│   ├── index.html               — estructura UI
│   ├── main.ts                  — lógica frontend, invoke a comandos Tauri
│   └── styles.css               — estilos ventana flotante
├── .github/
│   └── workflows/
│       └── release.yml          — CI/CD cross-platform builds
├── package.json
├── vite.config.ts
└── README.md                    — setup de desarrollo (Rust en Windows + WSL2)
```

**Servicios externos que el Flotante consume (sin modificar):**
- `http://localhost:3000/captures` — MindVault Server REST API
- `http://localhost:3000/health` — MindVault Server health check
- Binario `claude` en PATH del sistema

---

## Risks

| ID | Riesgo | Prob | Impacto | Mitigación |
|----|--------|------|---------|------------|
| R1 | Testing requiere Windows con Rust instalado — el entorno WSL2 no puede renderizar la ventana directamente | Alta | Alto | Documentar en README: Rust en Windows para dev/test, WSL2 solo para edición de código |
| R2 | Procesos huérfanos de `claude` si el timeout no mata explícitamente el proceso | Media | Medio | Usar `Child::kill()` explícito tras timeout; testear con prompts artificialmente lentas |
| R3 | Conflicto del hotkey global con otra aplicación del equipo | Media | Bajo | Elegir combinación poco común; documentar como configurable en Fase 2 |
| R4 | CI/CD requiere runner de macOS para builds `.dmg` (GitHub Actions cobra por minuto en macOS) | Baja | Medio | Configurar CI solo cuando haya distribución real; en Fase 1 builds manuales bastan |
| R5 | WebKit2GTK no instalado en Linux de algún miembro del equipo | Baja | Bajo | Documentar dependencia; los usuarios Linux son developers que pueden instalarlo |
| R6 | Cambio en endpoints del MindVault Server rompe el Flotante silenciosamente | Muy baja | Alto | Documentar versión mínima compatible; agregar verificación de health al inicio |

---

## Rollback Plan

El Flotante es una aplicación nueva e independiente. No modifica el MindVault Server ni ningún sistema existente. El rollback en cualquier punto es simplemente dejar de distribuir o ejecutar el Flotante — el Server sigue funcionando de forma autónoma.

Si una versión específica del Flotante tiene bugs:
1. El usuario deja de usarla y vuelve a capturar desde Claude Code directamente
2. Se distribuye la versión anterior si ya había una
3. No hay estado persistente en el Flotante que requiera migración — la persistencia vive en MindVault Server

---

## Dependencies

**Dependencias de runtime (usuario final):**
- Windows 10/11: WebView2 pre-instalado — nada extra
- macOS: WebKit pre-instalado — nada extra
- Linux: `libwebkit2gtk-4.1` del sistema (`sudo apt install libwebkit2gtk-4.1-dev`)
- MindVault Server corriendo en la URL configurada
- Claude Code instalado con MCP de MindVault configurado (para el flujo principal)

**Dependencias de desarrollo:**
- Rust stable + Cargo (instalar en Windows para testing, y en WSL2 opcionalmente para linting)
- Node.js v22+ (ya disponible en WSL2 via nvm)
- Tauri CLI: `cargo install tauri-cli`
- VS Build Tools (Windows, requerido por Rust en Windows)

**Dependencias de otros proyectos:**
- MindVault Server (`~/saasProjects/MindVault/mindvault-server/`) — debe estar en ejecución. No se modifica.

---

## Success Criteria

**Funcionales:**
- [ ] La app arranca como ventana always-on-top en Windows sin instalar nada adicional
- [ ] El usuario pega texto y presiona enviar — en < 30s aparece "Guardado" y el área se limpia
- [ ] Si `claude` no está en PATH o falla, el Flotante hace fallback a REST y muestra "Guardado como pendiente"
- [ ] Si el Server está caído, el fallback REST falla con mensaje claro de error (no silencioso)
- [ ] Los indicadores de estado (Server + Claude) reflejan correctamente el estado real del sistema
- [ ] El hotkey global trae la ventana al frente desde cualquier aplicación
- [ ] La ventana se puede minimizar al system tray y restaurar sin perder el texto ingresado
- [ ] La URL del Server se lee desde el archivo de configuración JSON en `app_data_dir`

**Técnicos:**
- [ ] El subprocess `claude` se mata explícitamente tras 30s de timeout (sin procesos huérfanos)
- [ ] El bundle de distribución pesa menos de 20MB en Windows
- [ ] La app arranca en menos de 1 segundo en Windows
- [ ] El código compila sin warnings en `cargo build --release`

**Distribución:**
- [ ] GitHub Actions genera `.exe` para Windows, `.dmg` para macOS y `.AppImage` para Linux en un solo workflow
- [ ] El instalador de Windows no requiere privilegios de administrador (instalación por usuario)
