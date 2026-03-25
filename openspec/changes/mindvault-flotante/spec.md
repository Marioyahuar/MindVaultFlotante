# Spec: MindVault Flotante

**ID:** mindvault-flotante
**Fecha:** 2026-03-24
**Tipo:** Spec completa (proyecto nuevo)
**Estado:** Borrador — pendiente de aprobación

---

## 1. Ventana Principal

**Requisitos:**

- La aplicación DEBE arrancar como una ventana always-on-top visible sobre todas las demás ventanas del sistema operativo.
- La ventana DEBE ser no redimensionable y de tamaño fijo.
- La ventana DEBE contener: un área de texto multilínea, un botón de envío, y dos indicadores de estado.
- El área de texto DEBE aceptar texto pegado desde el portapapeles y texto escrito manualmente.
- La ventana NO DEBE bloquear el foco de otras aplicaciones cuando está al frente — el usuario DEBE poder interactuar con otras apps sin cerrar el Flotante.
- La ventana DEBE poder minimizarse al system tray.
- Al restaurarse desde el system tray, la ventana DEBE mostrar el mismo contenido textual que tenía antes de minimizarse.
- La aplicación NO DEBE mostrar un ícono en la barra de tareas cuando está minimizada al tray.

**Escenarios:**

```
Escenario: Arranque inicial
  Given la aplicación se lanza por primera vez
  When el proceso de Tauri inicia
  Then la ventana aparece centrada en la pantalla
  And la ventana está visible sobre todas las demás ventanas
  And el área de texto está vacía y tiene el foco
  And los indicadores de estado muestran el estado real del sistema

Escenario: Minimizar al tray con texto en progreso
  Given el usuario tiene texto escrito en el área de entrada
  When el usuario hace click en el botón de minimizar al tray
  Then la ventana desaparece de la pantalla
  And el ícono del Flotante aparece en el system tray
  And el texto ingresado se conserva en memoria

Escenario: Restaurar desde tray
  Given la aplicación está minimizada al tray con texto en el área de entrada
  When el usuario hace click en el ícono del tray
  Then la ventana se muestra en el frente de todas las ventanas
  And el texto que estaba en el área de entrada sigue presente
```

---

## 2. Flujo de Captura — Happy Path (Claude disponible)

**Requisitos:**

- Cuando el usuario envía texto, el sistema DEBE verificar primero si `claude` está disponible en el PATH antes de intentar el subprocess.
- El sistema DEBE lanzar el subprocess `claude -p "Guardá esto como memoria en MindVault: {texto}"` de forma asíncrona.
- El sistema DEBE mostrar un estado visual de "procesando" mientras el subprocess está en ejecución.
- El texto enviado NO DEBE poder modificarse ni reenviarse mientras el subprocess está en ejecución.
- Si el subprocess termina con código de salida 0 dentro de los 30 segundos, el sistema DEBE mostrar el mensaje "Guardado" con indicación visual de éxito.
- Tras un guardado exitoso, el área de texto DEBE limpiarse automáticamente.
- El botón de envío DEBE volver a estar habilitado tras completarse el guardado.

**Escenarios:**

```
Escenario: Captura exitosa vía Claude
  Given el área de texto contiene "Reunión con Juan — decidimos usar PostgreSQL"
  And claude está disponible en el PATH
  And el MindVault Server está activo
  When el usuario presiona el botón de envío
  Then el botón de envío se deshabilita y aparece estado "Procesando..."
  And se lanza subprocess: claude -p "Guardá esto como memoria en MindVault: Reunión con Juan — decidimos usar PostgreSQL"
  And el subprocess termina con código 0 en menos de 30 segundos
  Then el área de texto se limpia
  And se muestra "Guardado" con indicación visual verde
  And el botón de envío se habilita nuevamente

Escenario: Intento de envío con área de texto vacía
  Given el área de texto está vacía o contiene solo espacios
  When el usuario presiona el botón de envío
  Then no se lanza ningún subprocess
  And no se hace ninguna llamada HTTP
  And el botón de envío permanece habilitado (no pasa nada)
```

---

## 3. Flujo de Captura — Fallback (Claude no disponible o falla)

**Requisitos:**

- Si `claude` no está en el PATH del sistema, el sistema DEBE ir directamente al fallback REST sin intentar el subprocess.
- Si el subprocess `claude` falla (código de salida distinto de 0), el sistema DEBE ir al fallback REST.
- Si el subprocess `claude` no termina dentro de 30 segundos, el sistema DEBE matar el proceso explícitamente y luego ir al fallback REST.
- El fallback DEBE hacer `POST {SERVER_URL}/captures` con el cuerpo `{ "raw_text": "{texto}" }`.
- Si el fallback REST responde con HTTP 201, el sistema DEBE mostrar "Guardado como pendiente" con indicación visual amarilla.
- Tras el fallback exitoso, el área de texto DEBE limpiarse automáticamente.
- Si tanto el subprocess como el fallback REST fallan, el sistema DEBE mostrar un mensaje de error claro y NO limpiar el área de texto (para que el usuario pueda reintentar o copiar el texto).
- El sistema NO DEBE perder el texto del usuario silenciosamente bajo ninguna circunstancia.

**Escenarios:**

```
Escenario: Fallback por Claude no disponible en PATH
  Given claude no está instalado o no está en el PATH del sistema
  And el MindVault Server está activo
  And el área de texto contiene texto válido
  When el usuario presiona el botón de envío
  Then no se intenta el subprocess
  And se hace POST {SERVER_URL}/captures con el texto
  And el servidor responde 201
  Then el área de texto se limpia
  And se muestra "Guardado como pendiente" con indicación visual amarilla

Escenario: Fallback por timeout de Claude (30 segundos)
  Given claude está en el PATH
  And el subprocess de claude lleva más de 30 segundos sin responder
  When se cumple el timeout
  Then el proceso claude se mata explícitamente (kill)
  And se hace POST {SERVER_URL}/captures con el texto original
  And el servidor responde 201
  Then el área de texto se limpia
  And se muestra "Guardado como pendiente" con indicación visual amarilla

Escenario: Fallback por error de Claude (código de salida != 0)
  Given claude está en el PATH
  And el subprocess de claude termina con código de salida distinto de 0
  When el subprocess termina con error
  Then se hace POST {SERVER_URL}/captures con el texto original
  And el resultado sigue el mismo comportamiento que el caso anterior

Escenario: Fallo total — Server también caído
  Given claude no está disponible
  And el MindVault Server está caído (no responde o responde 503)
  And el área de texto contiene texto válido
  When el usuario presiona el botón de envío
  Then se intenta el subprocess o se detecta que claude no está
  And se intenta el fallback REST y falla
  Then se muestra mensaje de error: "No se pudo guardar. Verificá que el Server esté activo."
  And el área de texto NO se limpia (el texto del usuario se preserva)
  And el botón de envío se habilita nuevamente para reintentar
```

---

## 4. Indicadores de Estado

**Requisitos:**

- La UI DEBE mostrar en todo momento dos indicadores permanentemente visibles: estado del Server y disponibilidad de Claude.
- El indicador del Server DEBE verificar `GET {SERVER_URL}/health` periódicamente.
- El intervalo de verificación del Server DEBERÍA ser de 10 segundos.
- El indicador de Claude DEBE verificar si el binario `claude` existe en el PATH del sistema.
- La verificación de Claude DEBERÍA realizarse al arranque y cada vez que el usuario intente enviar.
- Cada indicador DEBE tener al menos dos estados visuales distinguibles: activo (verde) e inactivo (rojo).
- Los indicadores DEBEN actualizarse de forma reactiva — si el Server cae mientras la app está abierta, el indicador DEBE cambiar a inactivo sin que el usuario haga nada.

**Escenarios:**

```
Escenario: Server activo al arranque
  Given la aplicación acaba de arrancar
  And el MindVault Server está corriendo y responde GET /health con 200
  When se completa la primera verificación de estado
  Then el indicador "Server" muestra estado activo (verde)

Escenario: Server se cae mientras la app está abierta
  Given el Flotante está abierto y el indicador del Server está en verde
  When el MindVault Server deja de responder (timeout o 503)
  Then en la próxima verificación periódica el indicador cambia a rojo
  And el cambio ocurre sin acción del usuario

Escenario: Claude no disponible
  Given el binario claude no está en el PATH del sistema
  When la aplicación verifica disponibilidad de Claude
  Then el indicador "Claude" muestra estado inactivo (rojo)
  And el flujo de captura usará directamente el fallback REST si el usuario envía
```

---

## 5. Hotkey Global

**Requisitos:**

- La aplicación DEBE registrar un atajo de teclado global activo en todo el sistema operativo, no solo cuando el Flotante tiene el foco.
- El hotkey DEBE traer la ventana del Flotante al frente y darle foco si estaba detrás de otras ventanas o minimizada al tray.
- El hotkey NO DEBE disparar el envío de texto automáticamente.
- Si el registro del hotkey falla (conflicto con otra app), la aplicación DEBE arrancar de todas formas y registrar el fallo en un log — no debe bloquearse.

**Escenarios:**

```
Escenario: Hotkey con ventana en segundo plano
  Given el Flotante está abierto pero detrás de otras ventanas
  And el usuario está trabajando en otra aplicación
  When el usuario presiona el hotkey global
  Then la ventana del Flotante pasa al frente
  And el área de texto recibe el foco

Escenario: Hotkey con ventana minimizada al tray
  Given el Flotante está minimizado al system tray
  When el usuario presiona el hotkey global
  Then la ventana del Flotante se muestra
  And pasa al frente de todas las ventanas
  And el área de texto recibe el foco

Escenario: Conflicto de hotkey al registrar
  Given otra aplicación ya usa el mismo atajo de teclado
  When el Flotante intenta registrar el hotkey al arrancar
  Then el registro falla silenciosamente (sin crash)
  And la aplicación continúa funcionando sin el hotkey
  And se registra el error en el log de la aplicación
```

---

## 6. Configuración

**Requisitos:**

- La URL del MindVault Server DEBE leerse desde un archivo de configuración JSON ubicado en el directorio estándar de datos de la aplicación del sistema operativo (`app_data_dir`).
- Si el archivo de configuración no existe al arrancar, el sistema DEBE crear uno con la URL por defecto `http://localhost:3000`.
- El sistema DEBE usar la URL configurada para todas las llamadas HTTP (health check y fallback REST).
- La configuración NO DEBE ser modificable desde la UI en Fase 1.

**Escenarios:**

```
Escenario: Primera ejecución sin archivo de configuración
  Given no existe archivo de configuración en app_data_dir
  When la aplicación arranca por primera vez
  Then se crea el archivo de configuración con URL por defecto: http://localhost:3000
  And la aplicación usa esa URL para health checks y fallback

Escenario: Configuración con URL personalizada
  Given el archivo de configuración contiene { "server_url": "http://192.168.1.10:3000" }
  When la aplicación arranca
  Then todas las llamadas HTTP se hacen a http://192.168.1.10:3000
  And el health check verifica http://192.168.1.10:3000/health

Escenario: Archivo de configuración corrupto o inválido
  Given el archivo de configuración existe pero tiene JSON inválido
  When la aplicación arranca
  Then se usa la URL por defecto http://localhost:3000
  And se registra el error en el log (sin crash)
```

---

## 7. Estados de la UI

**Requisitos:**

- La UI DEBE reflejar exactamente uno de los siguientes estados en todo momento: `idle`, `procesando`, `guardado`, `pendiente`, `error`.
- En estado `idle`: área de texto editable, botón de envío habilitado.
- En estado `procesando`: área de texto no editable, botón deshabilitado, indicación visual de actividad.
- En estado `guardado`: mensaje "Guardado" visible, área de texto limpia, botón habilitado. Transición automática a `idle` después de 2 segundos.
- En estado `pendiente`: mensaje "Guardado como pendiente" visible, área de texto limpia, botón habilitado. Transición automática a `idle` después de 2 segundos.
- En estado `error`: mensaje de error visible, área de texto con el texto original preservado, botón habilitado.
- La UI NO DEBE quedar permanentemente bloqueada en estado `procesando` — el timeout de 30s garantiza la transición.

**Escenarios:**

```
Escenario: Transición de estados en flujo exitoso
  Given la UI está en estado idle
  When el usuario envía texto y claude responde exitosamente
  Then la UI transiciona: idle → procesando → guardado → idle (en 2s)

Escenario: Transición de estados en flujo fallback
  Given la UI está en estado idle
  When el usuario envía texto y se usa el fallback REST exitosamente
  Then la UI transiciona: idle → procesando → pendiente → idle (en 2s)

Escenario: Transición de estados en fallo total
  Given la UI está en estado idle
  When el usuario envía texto y tanto claude como el fallback fallan
  Then la UI transiciona: idle → procesando → error
  And permanece en error hasta que el usuario modifica el texto o reintenta
```

---

## 8. Arranque y Cierre

**Requisitos:**

- La aplicación DEBE arrancar en menos de 1 segundo en Windows (desde doble-click hasta ventana visible).
- Al arrancar, la aplicación DEBE verificar inmediatamente el estado del Server y de Claude, antes de que el usuario pueda enviar texto.
- Al cerrar la ventana con texto en proceso (estado `procesando`), la aplicación DEBERÍA esperar a que el proceso actual complete antes de cerrar.
- La aplicación DEBE poder cerrarse completamente desde el menú del ícono en el tray.
- Al cerrarse, todos los subprocesses de `claude` en ejecución DEBEN terminarse explícitamente.

**Escenarios:**

```
Escenario: Cierre con subprocess en ejecución
  Given hay un subprocess de claude corriendo (estado procesando)
  When el usuario cierra la aplicación
  Then la aplicación mata el subprocess de claude
  And cierra sin dejar procesos huérfanos

Escenario: Cierre desde el tray
  Given la aplicación está minimizada al tray
  When el usuario hace click derecho en el ícono del tray y selecciona "Salir"
  Then la aplicación se cierra completamente
  And el ícono del tray desaparece
```
