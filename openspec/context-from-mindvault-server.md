# Contexto heredado de MindVault Server

Este documento resume las decisiones ya tomadas sobre el Flotante durante el diseño
y construcción de MindVault Server. Es la fuente de verdad para iniciar el SDD del Flotante.

**Fecha de referencia:** 2026-03-21
**Proyecto origen:** MindVault Server (archivado en ~/saasProjects/MindVault/openspec/changes/archive/2026-03-21-mindvault-mcp-memory-service/)

---

## Qué es el Flotante

Aplicación de escritorio pequeña, siempre visible sobre otras ventanas (always-on-top).
El usuario pega texto capturado de cualquier fuente (Gmail, WhatsApp Web, Teams, etc.)
y con un click o atajo de teclado lo envía a MindVault para ser guardado como memoria.

Es un **cliente** del MindVault Server. No tiene lógica de IA propia.

---

## Decisiones de diseño ya tomadas

### Stack
- **Tauri 2.x** — liviano (~10MB vs ~150MB de Electron), usa el webview del sistema operativo
- Backend en Rust, frontend en HTML/JS/TypeScript

### Flujo principal
1. Usuario pega texto en el Flotante
2. El Flotante lanza subprocess: `claude -p "Guardá esto como memoria en MindVault: {texto}"`
3. Claude Code (con MindVault MCP configurado) estructura el texto y llama `mindvault_save`
4. El Flotante muestra confirmación ("Guardado") y el texto desaparece

### Timeout y fallback
- Timeout del subprocess `claude -p`: **30 segundos**
- Si falla (claude no disponible, timeout, error): llamada directa a `POST /captures` del MindVault Server REST API
- El texto se guarda como `raw_capture` con `status: pending`
- El Flotante muestra "Guardado como pendiente" — el usuario puede procesar después desde Claude Code

### Indicadores de estado
El Flotante debe mostrar de forma visible:
- **Server activo / inactivo** — ping a `GET /health` del MindVault Server
- **Claude disponible / no disponible** — verifica que `claude` esté en el PATH del sistema

### Configuración
- URL del MindVault Server configurada en archivo `.env` o config local
- No requiere pantalla de configuración en Fase 1
- El usuario configura la URL una sola vez

---

## MindVault Server — lo que ya existe

El servidor ya está implementado, testeado y corriendo:
- **Código:** `~/saasProjects/MindVault/mindvault-server/`
- **Base de datos:** PostgreSQL en Neon (cloud)
- **MCP registrado** en `~/.claude.json` como servidor `"mindvault"`
- **REST API** disponible en `http://localhost:3000`
  - `POST /captures` — recibe raw_text + source_hint opcional, retorna `{ id, status: "pending" }`
  - `GET /health` — retorna `{ status: "ok", db: "ok" }` o HTTP 503

---

## Out of scope en Fase 1 del Flotante

- Autenticación
- Pantalla de configuración con UI
- Historial de capturas dentro del Flotante
- Sincronización con otros dispositivos
- Soporte para captura automática (sin acción del usuario)

---

## Proyecto hermano

El Flotante NO modifica MindVault Server. Solo lo consume.
Cualquier cambio necesario en el Server debe hacerse en su propio ciclo SDD
en `~/saasProjects/MindVault/`.
