# Tasks: mindvault-flotante
Estado: EN PROGRESO
Última actualización: 2026-03-24

---

## Fase 0: Repositorio y scaffold

- [x] 0.1 Ejecutar `git init` en `/home/marioyahuar/saasProjects/MindVaultFlotante/` y crear `.gitignore` para Rust + Node + Tauri (`target/`, `node_modules/`, `dist/`, `.env`)
- [ ] 0.2 Crear el proyecto Tauri 2.x con `cargo tauri init` desde Windows (PowerShell en `\\wsl$\Ubuntu\...\MindVaultFlotante`) — responder: nombre `MindVault Flotante`, identifier `com.mindvault.flotante`, frontend en `../dist`, dev URL `http://localhost:1420`
- [x] 0.3 Verificar que la estructura generada coincide con la del design.md: `src-tauri/`, `src/`, `package.json`, `vite.config.ts`
- [ ] 0.4 Instalar dependencias Node: `npm install` — verificar que `@tauri-apps/api` queda en `package.json`
- [x] 0.5 Agregar dependencias Rust en `src-tauri/Cargo.toml`: `reqwest` (features: `json`), `tokio` (features: `full`), `serde` (features: `derive`), `serde_json`, `tauri-plugin-global-shortcut`, `tauri-plugin-tray`
- [ ] 0.6 Verificar que `cargo build` compila sin errores desde Windows antes de tocar código <!-- Requiere: cargo build desde Windows -->

---

## Fase 1: Configuración y ventana base

- [x] 1.1 Editar `src-tauri/tauri.conf.json`: setear `alwaysOnTop: true`, `resizable: false`, `width: 420`, `height: 260`, `skipTaskbar: false`, `title: "MindVault"`
- [x] 1.2 Crear `src-tauri/capabilities/default.json` con permisos: `core:default`, `shell:allow-execute`, `global-shortcut:allow-register`, `global-shortcut:allow-unregister-all`, `tray:default`
- [x] 1.3 Crear `src-tauri/src/config.rs` con struct `Config { server_url: String }` y dos métodos: `cargar(app_handle: &AppHandle) -> Config` (lee `app_data_dir/config.json` o crea con default `http://localhost:3000`) y `url_captures(&self) -> String` y `url_health(&self) -> String`
- [x] 1.4 Agregar manejo de JSON corrupto en `config::cargar`: si `serde_json::from_str` falla, loguear el error y retornar config con valores por defecto (sin panic)
- [x] 1.5 Registrar `config.rs` como módulo en `src-tauri/src/main.rs` (`mod config;`)

---

## Fase 2: Módulo de estado del sistema

- [x] 2.1 Crear `src-tauri/src/estado.rs` con struct `EstadoSistema { server_activo: bool, claude_disponible: bool }` derivando `serde::Serialize`
- [x] 2.2 Implementar `estado::verificar_claude() -> bool`: usa `std::process::Command::new("where").arg("claude")` en Windows o `which` en Unix; retorna `true` si el exit code es 0
- [x] 2.3 Implementar `estado::verificar_server(url_health: &str) -> bool`: hace GET con `reqwest` y timeout de 5 segundos; retorna `true` solo si HTTP 200
- [x] 2.4 Implementar `estado::verificar(config: &Config) -> EstadoSistema`: llama a las dos funciones anteriores y ensambla la struct
- [x] 2.5 Implementar `estado::iniciar_loop_health_check(app: AppHandle, config: Config)`: spawnea tarea `tokio::spawn` que cada 10 segundos llama `verificar_server` y emite evento Tauri `"estado-servidor"` con payload `{ activo: bool }` usando `app.emit`
- [x] 2.6 Crear comando Tauri `obtener_estado` en `estado.rs` con atributo `#[tauri::command]` que llama `verificar(config)` y retorna `EstadoSistema`
- [x] 2.7 Registrar módulo y comando en `main.rs`

---

## Fase 3: Módulo de captura

- [x] 3.1 Crear `src-tauri/src/captura.rs` con enum `ResultadoCaptura { Guardado, Pendiente }` derivando `serde::Serialize`
- [x] 3.2 Implementar `captura::intentar_claude(texto: &str) -> Result<(), String>`: construye el prompt `format!("Guardá esto como memoria en MindVault: {}", texto)`, lanza subprocess `claude -p "{prompt}"` con `tokio::process::Command`, aplica `tokio::time::timeout(Duration::from_secs(30), ...)`, mata el proceso con `child.kill()` si hay timeout, retorna `Ok(())` si exit code 0 o `Err(motivo)` si falla
- [x] 3.3 Implementar `captura::intentar_fallback_rest(texto: &str, url: &str) -> Result<(), String>`: hace `POST url` con `reqwest`, body JSON `{ "raw_text": texto }`, timeout 10s; retorna `Ok(())` si HTTP 201, `Err(motivo)` en cualquier otro caso
- [x] 3.4 Implementar función orquestadora `captura::enviar(texto: &str, config: &Config) -> Result<ResultadoCaptura, String>`: primero verifica si claude está disponible (reusa `estado::verificar_claude()`), si sí intenta `intentar_claude`, si falla intenta `intentar_fallback_rest`, si también falla retorna `Err`; si claude no está disponible va directo a `intentar_fallback_rest`
- [x] 3.5 Crear comando Tauri `enviar_captura(texto: String) -> Result<String, String>` que llama `captura::enviar` y mapea `ResultadoCaptura::Guardado → "guardado"`, `ResultadoCaptura::Pendiente → "pendiente"`
- [x] 3.6 Registrar módulo y comando en `main.rs`

---

## Fase 4: Wiring en main.rs

- [x] 4.1 En `main.rs`, inicializar `Config` al arrancar con `config::cargar(&app_handle)` dentro del closure de `setup`
- [x] 4.2 Registrar plugins en el builder de Tauri: `tauri_plugin_tray::init()` y `tauri_plugin_global_shortcut::Builder::new().build()`
- [x] 4.3 Registrar el hotkey global `Ctrl+Shift+M` (Windows/Linux) / `Cmd+Shift+M` (macOS) en el setup: si falla el registro, loguear y continuar sin el hotkey (no crashear). El handler debe llamar `window.show()` + `window.set_focus()`
- [x] 4.4 Configurar el system tray en `main.rs`: ícono del tray, menú con opción "Mostrar" y "Salir"; "Mostrar" llama `window.show()` + `window.set_focus()`; "Salir" llama `app_handle.exit(0)`
- [x] 4.5 Interceptar el evento de cierre de ventana (`window.on_window_event`) para ocultar en lugar de cerrar cuando el usuario hace click en la X — la app solo se cierra completamente desde el menú del tray "Salir"
- [x] 4.6 Al arrancar, llamar `estado::iniciar_loop_health_check(app_handle.clone(), config.clone())` para iniciar las verificaciones periódicas
- [x] 4.7 Registrar todos los comandos en `invoke_handler`: `enviar_captura`, `obtener_estado`

---

## Fase 5: Frontend

- [x] 5.1 Reemplazar el contenido de `src/index.html` con la estructura de UI: `<textarea id="texto">`, `<button id="btn-enviar">`, `<div id="indicador-server">`, `<div id="indicador-claude">`, `<div id="mensaje-estado">`
- [x] 5.2 Crear `src/styles.css`: estilos para ventana compacta (sin scrollbar innecesaria), textarea que ocupa la mayor parte del espacio, botón de envío full-width, indicadores de estado en fila (verde/rojo con label), área de mensaje de estado con colores por tipo (verde/amarillo/rojo)
- [x] 5.3 Crear `src/main.ts` con la variable de estado `let estadoActual: EstadoUI` y la función `transicionar(nuevoEstado, mensaje?)` que actualiza el DOM según el estado y programa el retorno a `idle` tras 2000ms para los estados `guardado` y `pendiente`
- [x] 5.4 En `main.ts`, agregar el listener del botón de envío: valida que el textarea no esté vacío, llama `invoke("enviar_captura", { texto })`, maneja la promesa con `transicionar` según el resultado
- [x] 5.5 En `main.ts`, agregar la llamada inicial a `invoke("obtener_estado")` al cargar la página para mostrar estado correcto desde el arranque (antes de que el primer health check periódico ocurra)
- [x] 5.6 En `main.ts`, suscribirse al evento Tauri `"estado-servidor"` con `listen("estado-servidor", handler)` y actualizar el indicador del server en el DOM
- [ ] 5.7 Verificar en Windows que al presionar Enter en el textarea no se dispara el envío (el textarea es multilínea; Enter debe hacer salto de línea, no enviar). El envío es exclusivamente por botón.

---

## Fase 6: Tests unitarios Rust

- [x] 6.1 En `src-tauri/src/config.rs`, agregar módulo `#[cfg(test)]` con test: `config_default_cuando_no_existe_archivo` — verifica `Config::default()` con `server_url == "http://localhost:3000"`, `url_captures()` y `url_health()` correctos
- [ ] 6.2 En `config.rs`, agregar test: `config_json_valido_se_lee_correctamente` — escribe un JSON con URL custom en un path temporal y verifica que `cargar` lo lee bien
- [x] 6.3 En `config.rs`, agregar test: `config_json_corrupto_usa_default` — verifica que `serde_json::from_str` con JSON inválido falla correctamente
- [x] 6.4 En `src-tauri/src/estado.rs`, agregar test: `verificar_claude_retorna_bool_sin_panic` — verifica que no causa panic (valor depende del entorno)
- [x] 6.5 En `src-tauri/src/captura.rs`, agregar test con `mockito`: `fallback_rest_exitoso_cuando_server_retorna_201` — mockea `POST /captures` con respuesta 201 y verifica que `intentar_fallback_rest` retorna `Ok(())`
- [x] 6.6 En `captura.rs`, agregar test con `mockito`: `fallback_rest_falla_cuando_server_retorna_503` — mockea `POST /captures` con 503 y verifica que retorna `Err`
- [ ] 6.7 En `captura.rs`, agregar test: `enviar_va_directo_a_fallback_cuando_claude_no_disponible` — test manual requerido (depende del PATH)
- [x] 6.8 Agregar `mockito` a `[dev-dependencies]` en `Cargo.toml`

---

## Fase 7: Verificación manual en Windows

<!-- Requiere: verificación manual en Windows -->
- [ ] 7.1 Ejecutar `cargo tauri dev` desde PowerShell en el path del proyecto — verificar que la ventana aparece always-on-top y es no redimensionable
- [ ] 7.2 Pegar texto en el textarea y presionar el botón con MindVault Server activo y claude disponible — verificar transición `procesando → guardado` (verde) y limpieza del textarea
- [ ] 7.3 Pegar texto con MindVault Server activo pero sin claude en PATH (renombrar binario temporalmente) — verificar `procesando → pendiente` (amarillo)
- [ ] 7.4 Pegar texto con MindVault Server caído — verificar `procesando → error` (rojo) y que el texto se preserva
- [ ] 7.5 Verificar que los indicadores de estado cambian al estado correcto dentro de los 10 segundos posteriores a levantar/bajar el server
- [ ] 7.6 Presionar el hotkey `Ctrl+Shift+M` desde otra aplicación — verificar que el Flotante pasa al frente
- [ ] 7.7 Minimizar al tray con texto escrito, restaurar — verificar que el texto persiste
- [ ] 7.8 Hacer click en la X de la ventana — verificar que va al tray en lugar de cerrar
- [ ] 7.9 Cerrar desde el menú del tray "Salir" — verificar que la app cierra completamente

---

## Fase 8: README y distribución

- [x] 8.1 Crear `README.md` con secciones: Requisitos (por plataforma), Setup de desarrollo (Windows + WSL2), Cómo ejecutar en dev, Cómo buildear para distribución
- [x] 8.2 Crear `.github/workflows/release.yml` con matrix build para `windows-latest`, `macos-latest`, `ubuntu-22.04` — trigger en push de tag `v*.*.*`, upload de artifacts
- [ ] 8.3 Hacer primer commit con todo el código y ejecutar `cargo tauri build` desde Windows — verificar que el `.exe` resultante arranca en una máquina Windows limpia (sin Rust instalado) <!-- Requiere: cargo tauri build desde Windows -->
