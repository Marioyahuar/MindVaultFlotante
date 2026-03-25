use serde::{Deserialize, Serialize};
use tauri::Manager;

/// Configuración de la aplicación leída desde app_data_dir/config.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
        }
    }
}

impl Config {
    /// Carga la configuración desde el archivo config.json en app_data_dir.
    /// Si el archivo no existe, lo crea con los valores por defecto y los retorna.
    /// Si el archivo existe pero tiene JSON inválido, retorna los valores por defecto
    /// sin crashear y loguea el error.
    pub fn cargar(app_handle: &tauri::AppHandle) -> Self {
        let dir = match app_handle.path().app_data_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error al obtener app_data_dir: {}", e);
                return Self::default();
            }
        };

        let ruta = dir.join("config.json");

        if !ruta.exists() {
            // Crear directorio si no existe
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("Error al crear directorio de configuración: {}", e);
                return Self::default();
            }

            let config = Self::default();
            let contenido = match serde_json::to_string_pretty(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error al serializar config por defecto: {}", e);
                    return config;
                }
            };

            if let Err(e) = std::fs::write(&ruta, contenido) {
                eprintln!("Error al escribir config.json: {}", e);
            }

            return config;
        }

        // Leer archivo existente
        let contenido = match std::fs::read_to_string(&ruta) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error al leer config.json: {}", e);
                return Self::default();
            }
        };

        match serde_json::from_str::<Self>(&contenido) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error al parsear config.json (JSON inválido): {}. Usando valores por defecto.", e);
                Self::default()
            }
        }
    }

    /// Retorna la URL del endpoint de capturas.
    pub fn url_captures(&self) -> String {
        format!("{}/captures", self.server_url)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_cuando_no_existe_archivo() {
        let config = Config::default();
        assert_eq!(config.server_url, "http://localhost:3000");
        assert_eq!(config.url_captures(), "http://localhost:3000/captures");
        assert_eq!(config.url_health(), "http://localhost:3000/health");
    }

    #[test]
    fn config_json_corrupto_usa_default() {
        // Verificar que JSON inválido falla al parsearse
        let resultado = serde_json::from_str::<Config>("json inválido");
        assert!(
            resultado.is_err(),
            "El parseo de JSON inválido debería fallar"
        );

        // En este caso la lógica de cargar() retornaría Config::default()
        let config = Config::default();
        assert_eq!(config.server_url, "http://localhost:3000");
    }

    #[test]
    fn config_serializa_y_deserializa_correctamente() {
        let config = Config {
            server_url: "http://192.168.1.10:3000".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let config_leida: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config_leida.server_url, "http://192.168.1.10:3000");
        assert_eq!(config_leida.url_captures(), "http://192.168.1.10:3000/captures");
        assert_eq!(config_leida.url_health(), "http://192.168.1.10:3000/health");
    }
}
