import { invoke } from "@tauri-apps/api/core";

// Tipos
type EstadoUI = "idle" | "procesando" | "guardado" | "pendiente" | "error";

interface EstadoSistema {
  claude_disponible: boolean;
  mcp_configurado: boolean;
}

// Referencias al DOM
const textarea = document.getElementById("texto") as HTMLTextAreaElement;
const btnEnviar = document.getElementById("btn-enviar") as HTMLButtonElement;
const mensajeEstado = document.getElementById("mensaje-estado") as HTMLDivElement;
const puntoClaude = document.getElementById("punto-claude") as HTMLSpanElement;
const labelClaude = document.getElementById("label-claude") as HTMLSpanElement;
const puntoMcp = document.getElementById("punto-mcp") as HTMLSpanElement;
const labelMcp = document.getElementById("label-mcp") as HTMLSpanElement;

// Estado actual de la UI
let estadoActual: EstadoUI = "idle";
let timeoutRetornoIdle: number | null = null;

// Transicionar entre estados de la máquina de estados de la UI
function transicionar(estado: EstadoUI, mensaje?: string): void {
  estadoActual = estado;

  if (timeoutRetornoIdle !== null) {
    clearTimeout(timeoutRetornoIdle);
    timeoutRetornoIdle = null;
  }

  const bloqueado = estado === "procesando";
  textarea.disabled = bloqueado;
  btnEnviar.disabled = bloqueado;
  btnEnviar.classList.toggle("deshabilitado", bloqueado);
  textarea.classList.toggle("procesando", bloqueado);

  mensajeEstado.className = "mensaje-estado";
  mensajeEstado.textContent = "";

  switch (estado) {
    case "idle":
      mensajeEstado.classList.add("oculto");
      break;
    case "procesando":
      mensajeEstado.textContent = "Guardando...";
      break;
    case "guardado":
      mensajeEstado.classList.add("guardado");
      mensajeEstado.textContent = "Guardado";
      textarea.value = "";
      timeoutRetornoIdle = window.setTimeout(() => transicionar("idle"), 2000);
      break;
    case "pendiente":
      mensajeEstado.classList.add("pendiente");
      mensajeEstado.textContent = "Guardado como pendiente";
      textarea.value = "";
      timeoutRetornoIdle = window.setTimeout(() => transicionar("idle"), 2000);
      break;
    case "error":
      mensajeEstado.classList.add("error");
      mensajeEstado.textContent =
        mensaje ?? "No se pudo guardar. Verificá que Claude y MindVault MCP estén disponibles.";
      break;
  }
}

// Actualizar indicadores visuales del sistema
function actualizarIndicadores(estado: EstadoSistema): void {
  puntoClaude.className = "punto " + (estado.claude_disponible ? "activo" : "inactivo");
  labelClaude.textContent = estado.claude_disponible ? "Claude" : "Claude (no disponible)";

  puntoMcp.className = "punto " + (estado.mcp_configurado ? "activo" : "inactivo");
  labelMcp.textContent = estado.mcp_configurado ? "MindVault MCP" : "MindVault MCP (no configurado)";
}

// Listener del botón de envío
btnEnviar.addEventListener("click", async () => {
  const texto = textarea.value.trim();
  if (!texto) return;
  if (estadoActual === "procesando") return;

  transicionar("procesando");

  try {
    const resultado = await invoke<string>("enviar_captura", { texto });
    if (resultado === "guardado") {
      transicionar("guardado");
    } else {
      transicionar("pendiente");
    }
  } catch (error) {
    const mensajeError = typeof error === "string" ? error : undefined;
    transicionar("error", mensajeError);
  }
});

// Cargar estado inicial al arrancar
async function inicializar(): Promise<void> {
  try {
    const estado = await invoke<EstadoSistema>("obtener_estado");
    actualizarIndicadores(estado);
  } catch {
    actualizarIndicadores({ claude_disponible: false, mcp_configurado: false });
  }
}

inicializar();
