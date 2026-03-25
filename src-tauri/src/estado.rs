use serde::{Deserialize, Serialize};
use tauri::Emitter;

/// Estado actual del sistema desde la perspectiva del usuario:
/// si puede guardar memorias con IA (claude) y si el MCP está configurado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstadoSistema {
    pub claude_disponible: bool,
    pub mcp_configurado: bool,
}

/// Verifica si el binario `claude` está disponible en el PATH del sistema.
/// Usa `where` en Windows y `which` en Unix/macOS.
pub fn verificar_claude() -> bool {
    #[cfg(target_os = "windows")]
    let resultado = std::process::Command::new("where").arg("claude").output();

    #[cfg(not(target_os = "windows"))]
    let resultado = std::process::Command::new("which").arg("claude").output();

    match resultado {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Verifica si el MCP de MindVault está configurado en ~/.claude.json.
/// Lee el archivo y busca la entrada mcpServers.mindvault.
/// No hace ninguna llamada de red — es solo lectura de archivo local.
pub fn verificar_mcp_mindvault() -> bool {
    let home = match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        Ok(h) => h,
        Err(_) => return false,
    };

    let ruta = std::path::Path::new(&home).join(".claude.json");

    let contenido = match std::fs::read_to_string(&ruta) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let json: serde_json::Value = match serde_json::from_str(&contenido) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Verificar que mcpServers.mindvault existe y tiene un comando configurado
    json["mcpServers"]["mindvault"]["command"].is_string()
}

/// Verifica el estado completo del sistema.
pub fn verificar() -> EstadoSistema {
    EstadoSistema {
        claude_disponible: verificar_claude(),
        mcp_configurado: verificar_mcp_mindvault(),
    }
}

/// Inicia un loop en background que cada 30 segundos re-verifica el estado
/// y emite el evento "estado-sistema" con el resultado actualizado.
pub fn iniciar_loop_estado(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            let estado = verificar();

            if let Err(e) = app.emit("estado-sistema", &estado) {
                eprintln!("Error al emitir evento estado-sistema: {}", e);
            }
        }
    });
}

/// Comando Tauri: retorna el estado actual del sistema al frontend.
#[tauri::command]
pub fn obtener_estado() -> EstadoSistema {
    verificar()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estado_sistema_se_serializa_correctamente() {
        let estado = EstadoSistema {
            claude_disponible: true,
            mcp_configurado: false,
        };

        let json = serde_json::to_string(&estado).unwrap();
        assert!(json.contains("\"claude_disponible\":true"));
        assert!(json.contains("\"mcp_configurado\":false"));
    }

    #[test]
    fn estado_sistema_deserializa_correctamente() {
        let json = r#"{"claude_disponible":true,"mcp_configurado":true}"#;
        let estado: EstadoSistema = serde_json::from_str(json).unwrap();
        assert!(estado.claude_disponible);
        assert!(estado.mcp_configurado);
    }

    #[test]
    fn verificar_mcp_retorna_bool_sin_panic() {
        // Solo verificamos que no causa panic — el resultado depende del entorno
        let _resultado = verificar_mcp_mindvault();
    }

    #[test]
    fn verificar_claude_retorna_bool_sin_panic() {
        let _resultado = verificar_claude();
    }
}
