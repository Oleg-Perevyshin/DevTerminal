use lazy_static::lazy_static;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tauri::{command, AppHandle, Listener, State, Wry};
use tauri_plugin_serialplugin::commands::write;
use tauri_plugin_serialplugin::desktop_api::SerialPort;

use crate::{log, LogLevel, ReadDataResult};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SimpleSerialCommand {
  data: String,
  end_package: String,
}

lazy_static! {
  static ref DATA_BUFFERS: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Обрабатывает принятые данные по протоколу SimpleSerial и отправляет их через канал
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `port_path` - путь к серийному порту
/// * `on_event` - канал для отправки обработанных данных
///
/// # Returns
/// * `Ok(u32)` - ID события прослушивания
/// * `Err(String)` - ошибка при создании слушателя
#[command]
pub fn process_simple_serial(app: AppHandle<Wry>, port_path: String, on_event: Channel<String>) -> Result<u32, String> {
  let app_clone = app.clone();

  // Создаём буфер для накопления данных
  let buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
  let buffer_clone = buffer.clone();

  // Форматируем путь порта для использования в имени события
  let formatted_port_path = port_path
    .clone()
    .replace(".", "-")
    .replace("/", "-")
    .replace("\\", "-");
  log(
    LogLevel::Info,
    "process_simple_serial",
    format!("Форматированный путь порта: {}", formatted_port_path),
  );
  let listen_event_name = format!("plugin-serialplugin-read-{}", formatted_port_path);

  let event_id = app_clone.clone().listen(listen_event_name, move |event| {
    // Разбираем полезную нагрузку события
    if let Ok(payload) = serde_json::from_str::<ReadDataResult>(&event.payload()) {
      // Преобразуем байты в строку
      let data_str = match String::from_utf8(payload.data.clone()) {
        Ok(s) => s,
        Err(_) => String::from_utf8_lossy(&payload.data).to_string(),
      };
      log(LogLevel::Info, "process_simple_serial", format!("Данные в виде строки: {}", data_str));

      // Добавляем данные в буфер
      {
        let mut buffer_guard = buffer_clone.lock().unwrap();
        buffer_guard.push_str(&data_str);
        log(
          LogLevel::Info,
          "process_simple_serial",
          format!("Буфер обновлён, текущий размер: {}", buffer_guard.len()),
        );
        drop(buffer_guard);
      }

      // Обработка буфера
      let processed_remaining = {
        let buffer_guard = buffer_clone.lock().unwrap();
        let data = buffer_guard.clone();
        drop(buffer_guard);

        // Обрабатываем данные
        match process_simple_serial_data(&data, &on_event, &port_path, false) {
          Ok(remaining) => remaining,
          Err(e) => {
            log(LogLevel::Err, "process_simple_serial", format!("Ошибка обработки данных: {}", e));
            String::new()
          },
        }
      };

      // Сохраняем остаток в буфере
      let mut buffer_guard = buffer_clone.lock().unwrap();
      *buffer_guard = processed_remaining;
    }
  });

  Ok(event_id)
}

/// Вспомогательная функция для обработки данных SimpleSerial
///
/// # Arguments
/// * `data` - строка с данными для обработки
/// * `on_event` - канал для отправки обработанных данных
/// * `port_path` - путь к порту (для логирования)
/// * `interruption` - флаг режима прерывания
///
/// # Returns
/// * `Ok(String)` - оставшиеся необработанные данные
/// * `Err(String)` - ошибка обработки
fn process_simple_serial_data(data: &str, on_event: &Channel<String>, port_path: &str, interruption: bool) -> Result<String, String> {
  log(
    LogLevel::Info,
    "process_simple_serial_data",
    format!(
      "Начало обработки SimpleSerial данных для порта: {}, флаг прерывания: {}",
      port_path, interruption
    ),
  );

  let mut processed_data = String::new();
  let start_time = Instant::now();

  // Разбиваем на строки
  for chunk in data.split_inclusive(&['\n', '\r'][..]) {
    processed_data.push_str(chunk);

    // Если строка завершена символом \n или \r
    if chunk.ends_with('\n') || chunk.ends_with('\r') {
      let line = processed_data.trim_end_matches(&['\r', '\n'][..]);
      log(LogLevel::Info, "process_simple_serial_data", format!("Обнаружена завершённая строка: {}", line));

      if interruption {
        log(
          LogLevel::Info,
          "process_simple_serial_data",
          format!("Режим прерывания: добавление строки в буфер"),
        );
      } else {
        log(
          LogLevel::Info,
          "process_simple_serial_data",
          format!("Режим отправки: отправка строки через канал"),
        );
        on_event.send(line.to_string()).map_err(|e| e.to_string())?;
      }
      processed_data.clear();
    } else {
      // Проверяем, не прошло ли 5 секунд с последнего завершения строки
      if start_time.elapsed() >= Duration::from_secs(5) && !processed_data.is_empty() {
        log(
          LogLevel::Info,
          "process_simple_serial_data",
          format!("Таймаут 5 секунд превышен, отправка накопленных данных: {}", processed_data),
        );

        if interruption {
          log(
            LogLevel::Info,
            "process_simple_serial_data",
            format!("Режим прерывания: добавление таймаут-данных в буфер"),
          );
        } else {
          log(
            LogLevel::Info,
            "process_simple_serial_data",
            format!("Режим отправки: отправка таймаут-данных через канал"),
          );

          // Отправляем накопленные данные через канал
          on_event
            .send(processed_data.clone())
            .map_err(|e| e.to_string())?;
        }
        processed_data.clear();
      }
    }
  }

  log(
    LogLevel::Info,
    "process_simple_serial_data",
    format!("Оставшиеся данные после обработки: {}", processed_data),
  );
  // Возвращаем оставшуюся строку
  Ok(processed_data)
}

/// Отправляет команду по протоколу SimpleSerial в серийный порт
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `serial` - состояние серийного порта
/// * `port_path` - путь к серийному порту
/// * `sending_data` - JSON-объект с командой
///
/// # Returns
/// * `Ok(())` - команда успешно отправлена
/// * `Err(String)` - ошибка отправки
pub fn send_simple_serial_command(
  app: AppHandle,
  serial: State<'_, SerialPort<Wry>>,
  port_path: String,
  sending_data: serde_json::Value,
) -> Result<(), String> {
  log(
    LogLevel::Info,
    "send_simple_serial_command",
    format!("Начало отправки SimpleSerial команды на порт: {}", port_path),
  );

  // Разбираем JSON в структуру команды
  let command: SimpleSerialCommand = serde_json::from_value(sending_data).map_err(|e| {
    log(
      LogLevel::Err,
      "send_simple_serial_command",
      format!("Не удалось разобрать команду SimpleSerial: {}", e),
    );
    format!("Failed to parse simple serial command: {}", e)
  })?;

  log(
    LogLevel::Info,
    "send_simple_serial_command",
    format!("Разобранные данные команды: data={}, end_package={}", command.data, command.end_package),
  );

  // Отправляем команду в порт
  write(
    app.clone(),
    serial.clone(),
    port_path.clone(),
    format!("{}{}", command.data, command.end_package),
  )
  .map_err(|e| {
    log(LogLevel::Err, "send_simple_serial_command", format!("Не удалось записать данные в порт: {}", e));
    format!("Failed to write: {}", e)
  })?;

  log(LogLevel::Info, "send_simple_serial_command", format!("Команда SimpleSerial успешно отправлена"));
  Ok(())
}
