# Design: MindVault Flotante

**ID:** mindvault-flotante
**Fecha:** 2026-03-24
**Estado:** Borrador — pendiente de aprobación

---

## Visión general

MindVault Flotante es una aplicación Tauri 2.x con backend en Rust y frontend en HTML/TypeScript vanilla. La arquitectura sigue el modelo estándar de Tauri: el frontend vive en un webview y se comunica con el backend Rust exclusivamente a través de comandos `invoke`. Todo el trabajo pesado (subprocess, HTTP, configuración, PATH checks) ocurre en Rust; el frontend es responsable únicamente de la UI y el estado visual.

El backend Rust está organizado en cuatro módulos independientes: `captura` (orquesta el flujo claude → fallback), `estado` (health check periódico del server + verificación de claude), `config` (lectura/escritura del JSON de configuración) y `main` (registro de comandos Tauri y setup de plugins). Esta separación permite testear cada módulo de forma aislada sin levantar la aplicación completa.

El flujo de datos es unidireccional: el usuario dispara una acción en el frontend → el frontend llama `invoke` → Rust ejecuta la lógica → Rust devuelve un resultado tipado → el frontend actualiza el estado visual. Los eventos periódicos (health check) van en sentido inverso: Rust emite un evento Tauri que el frontend escucha.

---

## Decisiones de arquitectura

| Decisión | Alternativas consideradas | Justificación |
|----------|--------------------------|---------------|
| Frontend vanilla HTML/TS sin framework | Svelte, React | La UI tiene 1 textarea + 1 botón + 2 indicadores. Un framework agrega complejidad de build sin beneficio real. |
| Estado de UI como máquina de estados explícita en TS | Variables booleanas ad-hoc | 5 estados definidos en spec. Una máquina de estados evita combinaciones inválidas (ej: procesando + editable). Más fácil de testear. |
| `reqwest` async directo (no `tauri-plugin-http`) | `tauri-plugin-http`, `ureq` | Control total sobre timeouts. `tauri-plugin-http` añade una capa de abstracción innecesaria para 2 endpoints. `ureq` es síncrono, incompatible con async Tauri. |
| `tokio::time::timeout` + `Child::kill()` para subprocess | Thread con sleep + flag, `wait_timeout` de `wait-timeout` crate | Es el patrón idiomático de Tokio. `Child::kill()` garantiza que el proceso muere. Sleep+flag es propenso a race conditions. |
| Verificación de Claude con `which`/`where` en Rust | Intentar lanzar subprocess y ver si falla | Más rápido (sin subprocess). Da feedback inmediato al arrancar antes de que el usuario intente enviar. |
| Config en JSON en `app_data_dir` | `.env` en directorio de trabajo, variables de entorno | `app_data_dir` es el path estándar por OS para datos de usuario de apps de escritorio. `.env` es convención de apps web/servidor, no de escritorio. |
| Health check periódico en Rust con emisión de eventos | Health check desde el frontend (fetch) | El frontend no tiene acceso directo a HTTP. Centralizar en Rust evita duplicar lógica de config/URL. |
| System tray con `tauri-plugin-tray` | Implementación manual, ninguno | Plugin oficial de Tauri 2.x. Funciona en Windows/macOS/Linux de forma consistente. |
| Hotkey global con `tauri-plugin-global-shortcut` | X11/Win32 directo, ninguno | Plugin oficial. Maneja diferencias de plataforma automáticamente. Fallo de registro no crashea la app. |

---

## Estructura de archivos

```
MindVaultFlotante/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          — setup Tauri: plugins, comandos, tray, hotkey, evento de cierre
│   │   ├── captura.rs       — comando `enviar_captura`: claude subprocess + fallback REST
│   │   ├── estado.rs        — comando `obtener_estado` + loop de health check periódico
│   │   └── config.rs        — lectura/escritura de config.json en app_data_dir
│   ├── Cargo.toml
│   ├── build.rs             — generado por Tauri
│   └── tauri.conf.json      — ventana, identificador, plugins habilitados
│   └── capabilities/
│       └── default.json     — permisos: shell (subprocess), http, tray, global-shortcut
├── src/
│   ├── index.html           — estructura: textarea, botón, indicadores
│   ├── main.ts              — máquina de estados UI + event listeners + invoke calls
│   └── styles.css           — estilos: ventana compacta, colores de estado
├── .github/
│   └── workflows/
│       └── release.yml      — build matrix: windows-latest, macos-latest, ubuntu-latest
├── package.json             — scripts: dev, build; dependencias: @tauri-apps/api, vite
├── vite.config.ts           — bundler frontend
├── tsconfig.json
└── README.md                — setup para Windows (con WSL2) y standalone
```

---

## Flujo de datos

### Envío de captura (happy path)

```
[Frontend]                          [Backend Rust]                    [Externo]
    |                                     |                               |
    | invoke("enviar_captura", {texto})   |                               |
    |------------------------------------>|                               |
    |                                     | which claude / where claude   |
    |                                     |------------------------------>|
    |                                     |<-- ruta del binario           |
    |                                     |                               |
    |                                     | spawn: claude -p "..."        |
    |                                     |------------------------------>|
    |                                     |         (max 30s)             |
    |                                     |<-- exit code 0                |
    |                                     |                               |
    |<-- Ok("guardado")                   |                               |
    |                                     |                               |
[UI: estado "guardado" → idle en 2s]
```

### Envío con fallback

```
[Frontend]                          [Backend Rust]                    [MindVault Server]
    |                                     |                               |
    | invoke("enviar_captura", {texto})   |                               |
    |------------------------------------>|                               |
    |                                     | subprocess falla / timeout    |
    |                                     | child.kill() si timeout       |
    |                                     |                               |
    |                                     | POST /captures {raw_text}     |
    |                                     |------------------------------>|
    |                                     |<-- 201 {id, status:"pending"} |
    |                                     |                               |
    |<-- Ok("pendiente")                  |                               |
    |                                     |                               |
[UI: estado "pendiente" → idle en 2s]
```

### Health check periódico

```
[Backend Rust - loop cada 10s]      [MindVault Server]      [Frontend]
    |                                     |                       |
    | GET {server_url}/health             |                       |
    |------------------------------------>|                       |
    |<-- 200 {status:"ok"} / 503 / error  |                       |
    |                                     |                       |
    | emit("estado-servidor", {activo: bool})                     |
    |------------------------------------------------------>|     |
    |                               [UI actualiza indicador]      |
```

---

## Contratos de interfaz

### Comandos Tauri (Rust → Frontend vía invoke)

```rust
// Envía texto a MindVault. Resultado tipado.
// Retorna "guardado" si claude tuvo éxito.
// Retorna "pendiente" si se usó el fallback REST.
// Retorna Err(mensaje) si todo falló.
#[tauri::command]
async fn enviar_captura(texto: String) -> Result<String, String>

// Retorna el estado actual del sistema en el momento de la llamada.
// Usado al arrancar para la verificación inicial.
#[tauri::command]
async fn obtener_estado(app: AppHandle) -> EstadoSistema
```

```typescript
// Tipos en frontend
type ResultadoCaptura = "guardado" | "pendiente"

interface EstadoSistema {
  server_activo: boolean
  claude_disponible: boolean
}
```

### Eventos Tauri (Rust → Frontend vía emit)

```
evento: "estado-servidor"
payload: { activo: boolean }
// Emitido cada 10s por el loop de health check en Rust
```

### Estructura config.json

```json
{
  "server_url": "http://localhost:3000"
}
```

Ubicación por plataforma:
- Windows: `C:\Users\{usuario}\AppData\Roaming\mindvault-flotante\config.json`
- macOS: `~/Library/Application Support/mindvault-flotante/config.json`
- Linux: `~/.local/share/mindvault-flotante/config.json`

### Módulo `config.rs`

```rust
pub struct Config {
    pub server_url: String,
}

impl Config {
    pub fn cargar(app_handle: &AppHandle) -> Config  // lee o crea con defaults
    pub fn url_captures(&self) -> String             // "{server_url}/captures"
    pub fn url_health(&self) -> String               // "{server_url}/health"
}
```

### Módulo `captura.rs`

```rust
pub enum ResultadoCaptura {
    Guardado,    // claude subprocess exitoso
    Pendiente,   // fallback REST exitoso
}

pub async fn enviar(texto: &str, config: &Config) -> Result<ResultadoCaptura, String>
// Implementa: verificar claude → subprocess (30s timeout) → fallback REST → error
```

### Módulo `estado.rs`

```rust
pub struct EstadoSistema {
    pub server_activo: bool,
    pub claude_disponible: bool,
}

pub async fn verificar(config: &Config) -> EstadoSistema
// Hace GET /health + verifica PATH de claude

pub fn iniciar_loop_health_check(app: AppHandle, config: Config)
// Spawnea tarea tokio que emite "estado-servidor" cada 10s
```

---

## Máquina de estados del frontend

```
                    ┌─────────────────────────────────────┐
                    │              IDLE                    │
                    │  - textarea: editable               │
                    │  - botón: habilitado                 │
                    └─────────────┬───────────────────────┘
                                  │ usuario envía (texto no vacío)
                                  ▼
                    ┌─────────────────────────────────────┐
                    │           PROCESANDO                 │
                    │  - textarea: bloqueada               │
                    │  - botón: deshabilitado              │
                    │  - spinner visible                   │
                    └──────┬──────────────┬───────────────┘
               Ok("guardado")          Ok("pendiente")     Err(msg)
                    │                      │                   │
                    ▼                      ▼                   ▼
            ┌──────────────┐    ┌─────────────────┐  ┌───────────────┐
            │   GUARDADO   │    │    PENDIENTE     │  │    ERROR      │
            │ - área limpia│    │ - área limpia    │  │ - texto pres. │
            │ - msg verde  │    │ - msg amarillo   │  │ - msg rojo    │
            └──────┬───────┘    └────────┬─────────┘  └──────┬────────┘
               2 segundos           2 segundos          usuario edita
                    │                    │               o reintenta
                    └────────────────────┴───────────────────┘
                                         │
                                         ▼
                                       IDLE
```

Implementación en TypeScript:

```typescript
type EstadoUI = "idle" | "procesando" | "guardado" | "pendiente" | "error"

// Variable de estado global en main.ts
let estadoActual: EstadoUI = "idle"

function transicionar(nuevoEstado: EstadoUI, mensaje?: string): void
// Actualiza DOM según el estado. Las transiciones automáticas (guardado/pendiente → idle)
// se implementan con setTimeout de 2000ms dentro de esta función.
```

---

## Estrategia de testing

### Unit tests en Rust (`src-tauri/src/`)

| Módulo | Qué se testea |
|--------|--------------|
| `config.rs` | Creación de config por defecto, lectura de JSON válido, fallback en JSON inválido |
| `captura.rs` | Resultado cuando claude no está en PATH (mock), resultado cuando el fallback REST retorna 201, error cuando ambos fallan |
| `estado.rs` | `verificar()` con server respondiendo 200, con server devolviendo 503, con server sin responder (timeout) |

Los tests de `captura.rs` mockean la URL del server con `mockito` o `wiremock` para simular respuestas HTTP sin levantar el server real.

### Lo que NO se testea de forma automatizada

- La ventana gráfica (always-on-top, tray, hotkey) — se verifica manualmente en Windows.
- El subprocess real de `claude` — se testea con `claude` real en el entorno de desarrollo Windows.
- El comportamiento del timeout de 30s con claude real — se testea manualmente con un script que simula lentitud.

### Tests manuales requeridos antes de `sdd-verify`

1. Abrir Flotante en Windows → verificar que aparece always-on-top.
2. Enviar texto con MindVault Server activo y `claude` disponible → verificar "Guardado" verde.
3. Matar el server y enviar → verificar que el indicador cambia a rojo y aparece el mensaje de error.
4. Desactivar claude (renombrar binario temporalmente) → verificar fallback REST funciona.
5. Minimizar al tray con texto escrito → restaurar → verificar que el texto persiste.
6. Probar el hotkey global desde otra aplicación.

---

## Configuración de Tauri (`tauri.conf.json`)

```json
{
  "productName": "MindVault Flotante",
  "identifier": "com.mindvault.flotante",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [{
      "title": "MindVault",
      "width": 420,
      "height": 260,
      "resizable": false,
      "alwaysOnTop": true,
      "decorations": true,
      "skipTaskbar": false
    }],
    "trayIcon": {
      "iconPath": "icons/tray.png",
      "iconAsTemplate": true
    }
  },
  "plugins": {
    "global-shortcut": {},
    "shell": {
      "open": false
    }
  }
}
```

## Permisos (`capabilities/default.json`)

```json
{
  "identifier": "default",
  "description": "Permisos base del Flotante",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-execute",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister-all",
    "tray:default"
  ]
}
```

---

## Setup de desarrollo (WSL2 + Windows)

El flujo de trabajo divide responsabilidades entre WSL2 y Windows:

| Tarea | Dónde se ejecuta | Herramienta |
|-------|-----------------|-------------|
| Edición de código | WSL2 | VSCode + extensión Remote WSL |
| Control de versiones (git) | WSL2 | git en terminal WSL2 |
| Linting / format Rust | WSL2 | `rustfmt`, `clippy` (Rust instalado en WSL2) |
| `cargo tauri dev` (run + live reload) | **Windows** | PowerShell o Windows Terminal |
| `cargo tauri build` (distribución) | GitHub Actions | Windows/macOS/Linux runners |

**Rust debe instalarse en ambos lados:**
- WSL2: para linting, `rust-analyzer` en VSCode, compilación de utilidades
- Windows: para `cargo tauri dev` y testing de la app con ventana real

**Setup Windows (una vez):**
```powershell
# 1. Instalar Rust
winget install Rustlang.Rustup
# 2. Instalar VS Build Tools (requerido por Rust en Windows)
winget install Microsoft.VisualStudio.2022.BuildTools
# 3. Instalar Node.js
winget install OpenJS.NodeJS.LTS
# 4. Instalar Tauri CLI
cargo install tauri-cli
```

**Carpeta compartida WSL2 ↔ Windows:**
El proyecto vive en `/home/marioyahuar/saasProjects/MindVaultFlotante/` (filesystem WSL2).
Desde Windows es accesible como `\\wsl$\Ubuntu\home\marioyahuar\saasProjects\MindVaultFlotante\`.
Se puede abrir PowerShell en ese path y ejecutar `cargo tauri dev` directamente.

---

## CI/CD — GitHub Actions (`release.yml`)

```yaml
# Trigger: push de tag v*.*.*
# Matrix: 3 runners en paralelo
jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: windows-latest   # produce .exe + .msi
          - platform: macos-latest     # produce .dmg + .app
          - platform: ubuntu-22.04     # produce .AppImage + .deb
    steps:
      - checkout
      - setup Rust stable
      - setup Node.js
      - npm install
      - cargo tauri build
      - upload artifacts
```

En Fase 1 el CI no es requisito bloqueante — builds manuales desde Windows son suficientes para distribución al equipo pequeño.
