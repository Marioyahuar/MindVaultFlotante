# PRD: MindVault Flotante — Cliente de Captura de Escritorio

**ID:** mindvault-flotante
**Fecha:** 2026-03-24
**Estado:** Borrador — pendiente de aprobación del usuario

---

## Objetivo

MindVault ya tiene un servidor de memoria personal funcional con IA estructurada, pero
hoy capturar algo requiere abrir una terminal y escribir un comando. El Flotante resuelve
esa fricción: una ventana mínima siempre visible sobre el resto de aplicaciones donde el
usuario pega texto y lo envía a su memoria con un solo click o atajo de teclado.

El cliente aprovecha Claude Code (ya configurado con el MCP de MindVault) como motor de
estructuración. El Flotante no tiene lógica de IA propia — actúa como interfaz entre el
usuario y el stack existente. El resultado es que capturar una conversación de WhatsApp
Web, un correo o un fragmento de cualquier aplicación cuesta menos de 5 segundos.

---

## Usuarios afectados

| Rol | Impacto |
|-----|---------|
| Usuario final del sistema MindVault | Puede capturar texto desde cualquier contexto sin abrir terminal ni cambiar de flujo de trabajo |
| Sistema MindVault Server | Recibe capturas estructuradas (vía claude) o crudas (vía fallback REST) — sin cambios requeridos en el servidor |

---

## Criterios de aceptación

- [ ] La aplicación arranca como ventana always-on-top de menos de 10MB en Linux/macOS/Windows
- [ ] El usuario puede pegar texto en el área de entrada y presionar un botón o atajo de teclado para enviarlo
- [ ] El Flotante lanza `claude -p "..."` como subprocess con el texto recibido y espera la respuesta
- [ ] Si claude responde exitosamente en menos de 30 segundos, el Flotante muestra "Guardado" y limpia el área de entrada
- [ ] Si claude falla, está inactivo o supera los 30 segundos de timeout, el Flotante hace `POST /captures` a la REST API del Server
- [ ] Cuando el fallback REST funciona, el Flotante muestra "Guardado como pendiente"
- [ ] El Flotante muestra en todo momento un indicador visible del estado del MindVault Server (activo / inactivo), verificando `GET /health`
- [ ] El Flotante muestra en todo momento si `claude` está disponible en el PATH del sistema
- [ ] La URL del Server se configura mediante archivo `.env` o config local (sin pantalla de configuración en UI)
- [ ] La aplicación puede minimizarse al system tray y restaurarse sin perder el estado
- [ ] Existe un atajo de teclado global que trae el Flotante al frente si estaba minimizado o detrás de otras ventanas

---

## Out of scope

- Autenticación de cualquier tipo
- Pantalla de configuración con UI para cambiar la URL del Server u otras opciones
- Historial de capturas dentro de la interfaz del Flotante
- Sincronización con otros dispositivos
- Captura automática (sin acción explícita del usuario)
- Modificaciones al MindVault Server — el Flotante solo lo consume
- Soporte para múltiples servidores MindVault
- Arranque automático con el sistema operativo (el usuario lanza el Flotante manualmente)
- Detección automática del `source_hint` — se deja vacío en el fallback REST
- Campo separado de instrucciones adicionales — el usuario incluye todo el contexto en el input principal

---

## Dependencias conocidas

- **MindVault Server** corriendo en `http://localhost:3000` (o URL configurada) con los endpoints `POST /captures` y `GET /health` operativos
- **Claude Code** instalado y accesible en el PATH del sistema (`claude` binary), con el MCP de MindVault configurado en `~/.claude.json`
- **Tauri 2.x** — requiere Rust stable, Node.js y las dependencias del webview del sistema operativo (WebKit en Linux/macOS, WebView2 en Windows)

---

## Preguntas abiertas

Ninguna — todas resueltas antes de la aprobación del PRD.

---

## Notas adicionales

**Referencia técnica heredada del Server:**
El backend del Flotante en Rust usa `std::process::Command` para lanzar el subprocess de
`claude`. El frontend en TypeScript/HTML es minimalista: área de texto + botón de envío +
indicadores de estado. La comunicación frontend↔backend va por comandos Tauri (`invoke`).

El proyecto ya tiene `openspec/config.yaml` con el stack definido (Tauri 2.x, Rust stable,
TypeScript ^5.3) y `openspec/context-from-mindvault-server.md` con el detalle completo
de la interfaz REST del Server.

**Interfaz REST del Server (referencia):**
```
POST /captures   → { raw_text, source_hint? } → { id, status: "pending" }
GET  /health     → { status: "ok", db: "ok" } | HTTP 503
```
