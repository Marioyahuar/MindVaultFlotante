// Previene ventana de consola en Windows en modo release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod captura;
mod config;
mod estado;

use tauri::{
    menu::{Menu, MenuItem},
    Manager, WindowEvent,
};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Cargar configuración y registrarla en el state de Tauri
            // para que los comandos puedan acceder a ella
            let config = config::Config::cargar(app.handle());
            app.manage(config.clone());

            // Iniciar loop de verificación de estado en background
            estado::iniciar_loop_estado(app.handle().clone());

            // Configurar menú del system tray.
            // Los IDs deben ser explícitos para que on_menu_event pueda matchear.
            let menu_mostrar =
                MenuItem::with_id(app.handle(), "mostrar", "Mostrar", true, None::<&str>).unwrap();
            let menu_salir =
                MenuItem::with_id(app.handle(), "salir", "Salir", true, None::<&str>).unwrap();
            let menu =
                Menu::with_items(app.handle(), &[&menu_mostrar, &menu_salir]).unwrap();

            // Usar el ícono por defecto de la app para el tray
            let icono = app.default_window_icon().cloned();
            let mut tray_builder = TrayIconBuilder::new().menu(&menu);
            if let Some(icono) = icono {
                tray_builder = tray_builder.icon(icono);
            }
            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "mostrar" => {
                        if let Some(ventana) = app.get_webview_window("main") {
                            ventana.show().ok();
                            ventana.set_focus().ok();
                        }
                    }
                    "salir" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(ventana) = app.get_webview_window("main") {
                            ventana.show().ok();
                            ventana.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            // Registrar hotkey global: Ctrl+Shift+M (Windows/Linux) o Cmd+Shift+M (macOS)
            // Si falla el registro, loguear y continuar — la app sigue funcionando sin hotkey
            let app_handle_hotkey = app.handle().clone();
            if let Err(e) = app.global_shortcut().on_shortcut(
                "CommandOrControl+Shift+M",
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(ventana) = app_handle_hotkey.get_webview_window("main") {
                            ventana.show().ok();
                            ventana.set_focus().ok();
                        }
                    }
                },
            ) {
                eprintln!("No se pudo registrar el hotkey global: {}", e);
            }

            // Interceptar evento de cierre de ventana: X → ocultar al tray en lugar de cerrar
            // La app solo se cierra completamente desde el menú del tray "Salir"
            if let Some(ventana) = app.get_webview_window("main") {
                let ventana_clone = ventana.clone();
                ventana.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        ventana_clone.hide().ok();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            captura::enviar_captura,
            estado::obtener_estado,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar MindVault Flotante");
}
