use crate::poe_canable::process_poe_canable;
use crate::poe_serial::process_poe_serial;
use crate::simple_serial::process_simple_serial;

pub mod cmd;
pub mod convertation;
pub mod models;
pub mod protocols;

pub use cmd::*;
pub use convertation::*;
pub use models::*;
pub use protocols::*;

/// Точка входа в приложение Tauri
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init()) // Плагин для диалоговых окон
    .plugin(tauri_plugin_fs::init()) // Плагин для работы с файловой системой
    .plugin(tauri_plugin_opener::init()) // Плагин для открытия файлов
    .plugin(tauri_plugin_serialplugin::init()) // Плагин для работы с серийными портами
    .setup(|_app| {
      #[cfg(debug_assertions)] // Открываем devtools в режиме отладки
      {
        use tauri::Manager;
        let window = _app.get_webview_window("main").unwrap();
        window.open_devtools();
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      connect_serial_port, close_serial_port, process_data_sending, hard_restart, process_simple_serial, process_poe_serial, process_poe_canable
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
