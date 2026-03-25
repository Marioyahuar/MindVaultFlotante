use serde::Serialize;
use std::time::Duration;

/// Resultado posible de una captura exitosa.
#[derive(Debug, Serialize)]
pub enum ResultadoCaptura {
    /// El texto fue guardado vía subprocess de Claude.
    Guardado,
    /// El texto fue enviado al servidor como pendiente vía REST.
    Pendiente,
}

/// Intenta guardar el texto usando el subprocess `claude -p`.
/// Aplica un timeout de 30 segundos. Si se supera, mata el proceso.
/// Retorna Ok(()) si el subprocess termina con código 0.
pub async fn intentar_claude(texto: &str) -> Result<(), String> {
    if !crate::estado::verificar_claude() {
        return Err("claude no disponible en el PATH".to_string());
    }

    let prompt = format!(
        "Usá el tool mindvault_save del servidor MCP 'mindvault' para guardar este texto como memoria estructurada. No uses ningún otro sistema de memoria. El texto a guardar es: {}",
        texto
    );

    let mut child = tokio::process::Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .spawn()
        .map_err(|e| format!("Error al lanzar subprocess claude: {}", e))?;

    match tokio::time::timeout(Duration::from_secs(30), child.wait()).await {
        Ok(Ok(estado)) => {
            if estado.success() {
                Ok(())
            } else {
                Err(format!(
                    "claude retornó código de error: {}",
                    estado.code().unwrap_or(-1)
                ))
            }
        }
        Ok(Err(e)) => Err(format!("Error al esperar subprocess: {}", e)),
        Err(_timeout) => {
            // Timeout alcanzado — matar el proceso
            child.kill().await.ok();
            Err("Timeout: claude no respondió en 30 segundos".to_string())
        }
    }
}

/// Intenta guardar el texto usando el endpoint REST de capturas.
/// Hace POST con body JSON `{ "raw_text": texto }`.
/// Retorna Ok(()) si la respuesta es HTTP 201.
pub async fn intentar_fallback_rest(texto: &str, url_captures: &str) -> Result<(), String> {
    let cliente = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Error al crear cliente HTTP: {}", e))?;

    let body = serde_json::json!({ "raw_text": texto });

    let respuesta = cliente
        .post(url_captures)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error de red al conectar con el servidor: {}", e))?;

    let status = respuesta.status();

    if status == reqwest::StatusCode::CREATED {
        Ok(())
    } else {
        Err(format!("El servidor retornó código inesperado: {}", status))
    }
}

/// Función orquestadora del flujo de captura.
/// Orden: claude subprocess → fallback REST → error si ambos fallan.
pub async fn enviar(
    texto: &str,
    config: &crate::config::Config,
) -> Result<ResultadoCaptura, String> {
    match intentar_claude(texto).await {
        Ok(()) => return Ok(ResultadoCaptura::Guardado),
        Err(_) => {
            // Claude falló o no está disponible — intentar fallback REST
        }
    }

    match intentar_fallback_rest(texto, &config.url_captures()).await {
        Ok(()) => Ok(ResultadoCaptura::Pendiente),
        Err(e) => Err(e),
    }
}

/// Comando Tauri: recibe texto del frontend y ejecuta el flujo de captura completo.
/// Retorna "guardado" si Claude tuvo éxito, "pendiente" si se usó el fallback REST.
/// Retorna Err con mensaje descriptivo si todo falló.
#[tauri::command]
pub async fn enviar_captura(
    app: tauri::AppHandle,
    texto: String,
) -> Result<String, String> {
    use tauri::Manager;

    let config = {
        let state = app.state::<crate::config::Config>();
        state.inner().clone()
    };

    match enviar(&texto, &config).await {
        Ok(ResultadoCaptura::Guardado) => Ok("guardado".to_string()),
        Ok(ResultadoCaptura::Pendiente) => Ok("pendiente".to_string()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fallback_rest_exitoso_retorna_ok() {
        let mut servidor = mockito::Server::new_async().await;

        let _mock = servidor
            .mock("POST", "/captures")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":1,"status":"pending"}"#)
            .create_async()
            .await;

        let url = format!("{}/captures", servidor.url());
        let resultado = intentar_fallback_rest("texto de prueba", &url).await;

        assert!(
            resultado.is_ok(),
            "Debería retornar Ok cuando el servidor responde 201"
        );
    }

    #[tokio::test]
    async fn fallback_rest_server_caido_retorna_err() {
        let mut servidor = mockito::Server::new_async().await;

        let _mock = servidor
            .mock("POST", "/captures")
            .with_status(503)
            .with_body("Service Unavailable")
            .create_async()
            .await;

        let url = format!("{}/captures", servidor.url());
        let resultado = intentar_fallback_rest("texto de prueba", &url).await;

        assert!(
            resultado.is_err(),
            "Debería retornar Err cuando el servidor responde 503"
        );
    }

    // NOTA: Este test verifica el comportamiento cuando claude no está en PATH.
    // Como el resultado de verificar_claude() depende del entorno, este caso
    // se verifica indirectamente: si claude no está disponible, intentar_claude
    // retornará Err inmediatamente sin lanzar subprocess.
    // Test manual requerido: renombrar el binario claude y verificar que se usa
    // el fallback REST correctamente.
    #[test]
    fn resultado_captura_serializa_correctamente() {
        let guardado = serde_json::to_string(&ResultadoCaptura::Guardado).unwrap();
        let pendiente = serde_json::to_string(&ResultadoCaptura::Pendiente).unwrap();
        assert_eq!(guardado, "\"Guardado\"");
        assert_eq!(pendiente, "\"Pendiente\"");
    }
}
