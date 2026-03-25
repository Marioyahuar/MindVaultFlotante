# Exploración: MindVault Flotante

## Contexto del cambio

MindVault Flotante es una aplicación de escritorio pequeña, siempre visible sobre otras ventanas (always-on-top), que permite al usuario capturar texto desde cualquier contexto (Gmail, WhatsApp Web, Teams, etc.) y enviarlo a MindVault Server para ser guardado como memoria personal.

El cambio no es evolutivo sino de **adición de un nuevo cliente** al ecosistema MindVault existente. MindVault Server ya está en producción con su propia arquitectura estable. El Flotante es complementario: no modifica el servidor, solo lo consume.

**Fecha de referencia del contexto heredado:** 2026-03-21
**Proyecto servidor:** ~/saasProjects/MindVault/ (archivado tras completarse el SDD)
**Proyecto cliente:** ~/saasProjects/MindVaultFlotante/ (nuevo proyecto en fase SDD)

---

## Estado actual del proyecto

### Estructura de directorios

El proyecto MindVaultFlotante existe solo con artefactos SDD — sin código de aplicación aún:
```
/home/marioyahuar/saasProjects/MindVaultFlotante/
└── openspec/
    ├── config.yaml
    ├── context-from-mindvault-server.md
    └── changes/mindvault-flotante/
        └── PRD.md
```

### Configuración establecida

**Stack definido en `openspec/config.yaml`:**
- Framework: Tauri 2.x
- Backend: Rust (stable)
- Frontend: TypeScript 5.3+
- Reglas: código y comentarios en español, variables/funciones en inglés

---

## MindVault Server — interfaz consumida

### POST /captures

Recibe texto crudo y lo persiste como captura pendiente.

**Request body:**
```json
{
  "raw_text": "string",      // REQUERIDO, no vacío
  "source_hint": "string?"   // OPCIONAL
}
```

**Response 201:**
```json
{ "id": "string", "status": "pending" }
```

**Response 400:**
```json
{ "error": "El campo \"raw_text\" es requerido y no puede estar vacío." }
```

**Response 503:**
```json
{ "error": "Base de datos no disponible" }
```

Código fuente: `mindvault-server/src/api/routes.ts:14-34`

### GET /health

**Response 200:**
```json
{ "status": "ok", "db": "ok" }
```

**Response 503:**
```json
{ "status": "degraded", "db": "error" }
```

Código fuente: `mindvault-server/src/api/routes.ts:40-49`

### Tipos de datos relevantes

```typescript
// Tabla raw_captures — para el fallback
RawCapture {
  id: string;
  raw_text: string;
  source_hint: string | null;
  status: "pending" | "processing" | "completed";
  captured_at: Date;
}
```

Código fuente: `mindvault-server/src/types.ts`

---

## Entorno de desarrollo

| Herramienta | Estado | Versión |
|-------------|--------|---------|
| Node.js | Instalado (nvm) | v22.11.0 |
| Claude | Instalado, en PATH | 2.1.81 |
| Rust / cargo | **NO instalado** | — |
| pkg-config | NO instalado | — |
| WebKit2GTK | NO verificado | — |
| OS | WSL2 (Linux) | 5.15.167.4 |

**Acciones requeridas antes de compilar:**
1. Instalar Rust via rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Instalar dependencias de sistema en WSL2:
   ```bash
   sudo apt-get install pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
   ```
3. Instalar Tauri CLI: `cargo install tauri-cli`

---

## Patrones Tauri 2.x relevantes

### Estructura típica del proyecto

```
MindVaultFlotante/
├── src-tauri/
│   ├── src/
│   │   └── main.rs        # Punto de entrada Rust
│   ├── Cargo.toml
│   └── tauri.conf.json    # Configuración ventana, permisos, plugins
├── src/                   # Frontend HTML/TypeScript
│   ├── index.html
│   └── main.ts
├── package.json
└── vite.config.ts
```

### Comunicación frontend ↔ backend

```typescript
// Frontend TypeScript → Backend Rust
import { invoke } from '@tauri-apps/api/core';
const result = await invoke('enviar_captura', { texto: '...' });
```

```rust
// Backend Rust — define el comando
#[tauri::command]
async fn enviar_captura(texto: String) -> Result<String, String> { ... }

// Registra en main
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![enviar_captura])
```

### Ventana always-on-top

En `tauri.conf.json`:
```json
{
  "windows": [{
    "alwaysOnTop": true,
    "resizable": false,
    "width": 450,
    "height": 280
  }]
}
```

### Subprocess con timeout (Rust)

```rust
use tokio::time::{timeout, Duration};
use std::process::Command;

let resultado = timeout(
    Duration::from_secs(30),
    tokio::task::spawn_blocking(|| {
        Command::new("claude")
            .arg("-p")
            .arg(&prompt)
            .output()
    })
).await;
```

### System tray y hotkey global

- System tray: plugin `tauri-plugin-tray` (oficial Tauri 2.x)
- Hotkey global: plugin `tauri-plugin-global-shortcut` (oficial Tauri 2.x)
- El hotkey en Fase 1 solo trae la ventana al frente (no auto-envía)

### HTTP client en Rust

Plugin oficial `tauri-plugin-http` — wrapper sobre `reqwest`. Alternativa: `reqwest` directo en Cargo.toml sin plugin (más control, misma librería subyacente).

---

## Comparación de enfoques

### Frontend: HTML/JS vanilla vs. framework

| Enfoque | Pros | Contras | Complejidad |
|---------|------|---------|-------------|
| HTML + TypeScript vanilla | Sin dependencias, tamaño mínimo, fácil de entender | Más verboso para manejo de estado | Baja |
| Svelte | Reactivo, muy liviano en bundle, sintaxis limpia | Una dependencia más, curva de aprendizaje | Baja-Media |
| React/Vue | Ecosistema amplio | Overkill para una UI tan simple, bundle más grande | Media |

**Recomendación:** HTML + TypeScript vanilla. La UI es un textarea + botón + dos indicadores. No justifica un framework.

### HTTP client en Rust: plugin vs. directo

| Enfoque | Pros | Contras | Complejidad |
|---------|------|---------|-------------|
| `tauri-plugin-http` | Integrado con permisos Tauri, configuración centralizada | Abstracción extra | Baja |
| `reqwest` directo | Control total, sin intermediario, ampliamente documentado | Requiere configurar permisos Tauri manualmente | Baja |

**Recomendación:** `reqwest` con feature `blocking` desactivado (async). Más control sobre timeouts y errores. El plugin es innecesario para un uso tan específico.

### Lectura de config: `.env` vs. archivo JSON

| Enfoque | Pros | Contras | Complejidad |
|---------|------|---------|-------------|
| `.env` en directorio de la app | Convención conocida por developers | Requiere `dotenv` crate | Baja |
| Archivo JSON en app data dir | Path estándar por OS, Tauri tiene API para ubicarlo | Más código para leer/escribir | Baja-Media |

**Recomendación:** JSON en el directorio de datos de la app (`app_data_dir` de Tauri). Es el patrón estándar para apps de escritorio. El `.env` es más adecuado para aplicaciones web/servidor.

---

## Riesgos identificados

### Alta severidad

**R1 — Rust no instalado**
Tauri requiere Rust stable para compilar. Sin él, el proyecto no arranca.
Mitigación: documentar setup en README antes de comenzar implementación.

**R2 — WebKit en WSL2**
Tauri en WSL2 requiere WebKit2GTK y potencialmente X11/Wayland forwarding. Sin display server, la ventana no puede renderizarse.
Mitigación: testear en entorno con display disponible; documentar alternativas (Linux nativo, VM).

### Media severidad

**R3 — Procesos huérfanos de Claude**
Si el subprocess de `claude -p` supera el timeout, debe matarse explícitamente. Sin `child.kill()`, el proceso sigue corriendo en background.
Mitigación: usar `Child::kill()` tras timeout; testear con prompts lentas.

**R4 — Conflicto de hotkey global**
El atajo elegido puede estar en uso por otra app. Sin configurabilidad, el usuario no puede cambiarlo.
Mitigación: elegir combinación poco común (ej: Ctrl+Shift+Alt+M); documentar como configurable en Fase 2.

### Baja severidad

**R5 — Cambios en API del Server**
Si MindVault Server modifica `/captures` o `/health`, el Flotante se rompe silenciosamente.
Mitigación: documentar "compatible con MindVault Server ≥ v1.0.0"; versionado explícito.

**R6 — Usuario no distingue guardado estructurado vs. pendiente**
Si el usuario ve "Guardado" sin más detalle, no sabe si fue procesado por Claude o quedó pendiente.
Mitigación: mensajes diferenciados con color — "Guardado" (verde, Claude) vs. "Guardado como pendiente" (amarillo, fallback).

---

## Recomendación

El proyecto está listo para pasar a `/sdd-propose` y luego `/sdd-spec`.

**Decisiones técnicas recomendadas:**
- Frontend: HTML + TypeScript vanilla (sin framework)
- HTTP client: `reqwest` async directo
- Config: JSON en `app_data_dir` de Tauri
- Subprocess timeout: `tokio::time::timeout` con `child.kill()` explícito
- Plugins Tauri necesarios: `tauri-plugin-tray`, `tauri-plugin-global-shortcut`

**Precondiciones antes de `/sdd-apply`:**
- [ ] Rust instalado en máquina de desarrollo
- [ ] Dependencias de sistema instaladas (WebKit2GTK, pkg-config)
- [ ] MindVault Server corriendo y respondiendo en `GET /health`

**Archivos de referencia:**
- `openspec/config.yaml` — stack y reglas del proyecto
- `openspec/context-from-mindvault-server.md` — interfaz REST del Server
- `openspec/changes/mindvault-flotante/PRD.md` — criterios de aceptación
- `mindvault-server/src/api/routes.ts` — implementación de endpoints consumidos
- `mindvault-server/src/types.ts` — tipos de datos de referencia
