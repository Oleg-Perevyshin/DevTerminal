/* src-tauri\src\cmd.rs */
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;
use tauri::{command, AppHandle, Emitter, Listener, State, Wry};
use tauri_plugin_serialplugin::commands::{close, force_close, open, start_listening, stop_listening, write, write_data_terminal_ready, write_request_to_send};
use tauri_plugin_serialplugin::desktop_api::SerialPort;

use crate::convertation::*;
use crate::models::*;
use crate::poe_canable::send_poe_canable_command;
use crate::poe_serial::send_poe_serial_command;
use crate::simple_serial::send_simple_serial_command;

/// Уровень логирования приложения.
/// 0: none, 1: error, 2: warning, 3: info, 4: debug
static APP_LOG_LEVEL: AtomicU8 = AtomicU8::new(3);

/// Перечисление уровней логирования.
#[derive(Debug)]
pub enum LogLevel {
  Err,
  Warn,
  Info,
}

impl From<u8> for LogLevel {
  /// Преобразует числовое значение в уровень логирования.
  /// 1 -> Err, 2 -> Warn, остальные -> Info
  fn from(level: u8) -> Self {
    match level {
      1 => LogLevel::Err,
      2 => LogLevel::Warn,
      _ => LogLevel::Info,
    }
  }
}

/// Функция логирования с поддержкой фильтрации по уровню.
///
/// # Arguments
/// * `level` - уровень логирования (Err, Warn, Info)
/// * `label` - метка/название функции, откуда вызывается лог
/// * `args` - строка с сообщением для лога
pub fn log(level: LogLevel, label: &str, args: String) {
  let log_level_val = match level {
    LogLevel::Err => 1,
    LogLevel::Warn => 2,
    LogLevel::Info => 3,
  };

  let current_level = APP_LOG_LEVEL.load(Ordering::Relaxed);

  if current_level == 0 {
    return;
  } else if current_level == 1 && log_level_val != 1 {
    return;
  } else if current_level == 2 && (log_level_val > 2) {
    return;
  } else if current_level == 3 && (log_level_val > 3) {
    return;
  }

  let level_str = match level {
    LogLevel::Err => "ERR",
    LogLevel::Warn => "WARN",
    LogLevel::Info => "INFO",
  };

  let timestamp = chrono::Local::now().time().format("%H:%M:%S").to_string();
  println!("[{}] {}: {} | {}", timestamp, level_str, label, args);
}

/// Подключается к серийному порту с заданной конфигурацией.
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `serial` - состояние серийного порта
/// * `config` - конфигурация подключения (путь, скорость, протокол и т.д.)
///
/// # Returns
/// * `Ok(String)` - путь к подключенному порту
/// * `Err(String)` - ошибка подключения
#[command]
pub async fn connect_serial_port(app: AppHandle<Wry>, serial: State<'_, SerialPort<Wry>>, config: SerialConfig) -> Result<String, String> {
  log(LogLevel::Info, "connect_serial_port", format!("Попытка подключения к порту: {}", config.path));

  /* Открытие порта */
  match open(
    app.clone(),
    serial.clone(),
    config.path.clone(),
    config.baud_rate.clone(),
    data_bits_from_u32(config.data_bits.clone()),
    flow_control_from_u32(config.flow_control.clone()),
    parity_from_u32(config.parity.clone()),
    stop_bits_from_u32(config.stop_bits.clone()),
    config.timeout.clone(),
  ) {
    Ok(_) => {
      log(LogLevel::Info, "connect_serial_port", format!("Порт {} успешно открыт", config.path));
    },
    Err(e) => {
      log(
        LogLevel::Err,
        "connect_serial_port",
        format!("Не удалось открыть порт {}: {}", config.path.clone(), e),
      );
      if let Err(e2) = app
        .clone()
        .emit("app-status", format!("Failed to open port {}: {}", config.path.clone(), e))
      {
        eprintln!("Failed to emit data: {}", e2);
      }
    },
  }
  tokio::time::sleep(Duration::from_millis(100)).await;

  /* Открытие CAN порта*/
  if config.protocol == "POECanable" || config.protocol == "POECanableFD" {
    log(
      LogLevel::Info,
      "connect_serial_port",
      format!("Инициализация CAN протокола: {}", config.protocol),
    );
    write(app.clone(), serial.clone(), config.path.clone().to_string(), "C\r".to_string()).map_err(|e| e.to_string())?;
    if config.protocol == "POECanable" {
      write(
        app.clone(),
        serial.clone(),
        config.path.clone().to_string(),
        format!("{}\r", config.can_bitrate.unwrap()),
      )
      .map_err(|e| {
        log(LogLevel::Err, "connect_serial_port", format!("Не удалось отправить команду 'C': {}", e));
        e.to_string()
      })?;
    } else {
      /* ПОМЕНЯТЬ СТРОКУ  */
      write(
        app.clone(),
        serial.clone(),
        config.path.clone().to_string(),
        format!("{}\r", config.canfd_bitrate.unwrap()),
      )
      .map_err(|e| {
        log(
          LogLevel::Err,
          "connect_serial_port",
          format!("Не удалось отправить команду скорости CANFD: {}", e),
        );
        e.to_string()
      })?;
      write(
        app.clone(),
        serial.clone(),
        config.path.clone().to_string(),
        format!("{}\r", config.canfd_data_bitrate.unwrap()),
      )
      .map_err(|e| {
        log(
          LogLevel::Err,
          "connect_serial_port",
          format!("Не удалось отправить команду скорости данных CANFD: {}", e),
        );
        e.to_string()
      })?;
    }
    let init_commands = vec!["M0\r", "A0\r", "O\r"];
    log(LogLevel::Info, "connect_serial_port", format!("Отправка команд инициализации для CAN"));

    for command in init_commands {
      write(app.clone(), serial.clone(), config.path.clone(), command.to_string()).map_err(|e| {
        log(
          LogLevel::Err,
          "connect_serial_port",
          format!("Не удалось отправить команду инициализации {}: {}", command, e),
        );
        e.to_string()
      })?;
    }
  }

  /* Установка флагов DTR и RTS */
  log(LogLevel::Info, "connect_serial_port", format!("Установка флагов DTR и RTS в false"));

  let _ = write_data_terminal_ready(app.clone(), serial.clone(), config.path.to_string().clone(), false).map_err(|e| {
    log(LogLevel::Err, "connect_serial_port", format!("Не удалось установить флаг DTR: {}", e));
    e.to_string()
  });
  let _ = write_request_to_send(app.clone(), serial.clone(), config.path.to_string().clone(), false).map_err(|e| {
    log(LogLevel::Err, "connect_serial_port", format!("Не удалось установить флаг RTS: {}", e));
    e.to_string()
  });

  /* Начало прослушивания порта */
  log(LogLevel::Info, "connect_serial_port", format!("Начало прослушивания порта: {}", config.path));

  match start_listening(app.clone(), serial.clone(), config.path.clone().to_string(), None, None) {
    Ok(_) => {
      log(
        LogLevel::Info,
        "connect_serial_port",
        format!("Прослушивание успешно начато на порту: {}", config.path),
      );

      if let Err(e) = app
        .clone()
        .emit("app-status", format!("Port {} opened successfully", config.path.clone()))
      {
        eprintln!("Failed to emit data: {}", e);
      }
    },
    Err(e) => {
      log(
        LogLevel::Err,
        "connect_serial_port",
        format!("Не удалось начать прослушивание порта {}: {}", config.path.clone(), e),
      );

      if let Err(e2) = app
        .clone()
        .emit("app-status", format!("Failed to open port {}: {}", config.path.clone(), e))
      {
        eprintln!("Failed to emit data: {}", e2);
      }
    },
  }
  log(
    LogLevel::Info,
    "connect_serial_port",
    format!("Процесс подключения завершён, возвращён путь: {}", config.path),
  );

  Ok(config.path)
}

/// Закрывает подключенный серийный порт.
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `serial` - состояние серийного порта
/// * `path` - путь к закрываемому порту
/// * `event_id` - ID события прослушивания для отключения
/// * `can_protocol` - флаг, указывающий, используется ли CAN протокол
///
/// # Returns
/// * `Ok(())` - успешно закрыто
/// * `Err(String)` - ошибка закрытия
#[command]
pub async fn close_serial_port(app: AppHandle<Wry>, serial: State<'_, SerialPort<Wry>>, path: String, event_id: u32, can_protocol: bool) -> Result<(), String> {
  log(
    LogLevel::Info,
    "close_serial_port",
    format!("Начало закрытия порта: {}, CAN протокол: {}", path, can_protocol),
  );

  /* Закрытие CAN порта */
  if can_protocol == true {
    log(LogLevel::Info, "close_serial_port", format!("Отправка команды 'C' для закрытия CAN порта"));

    write(app.clone(), serial.clone(), path.clone().to_string(), "C\r".to_string()).map_err(|e| {
      log(LogLevel::Err, "close_serial_port", format!("Не удалось отправить команду 'C': {}", e));
      e.to_string()
    })?;
  }

  /* Отключение слушателей  */
  log(LogLevel::Info, "close_serial_port", format!("Отключение слушателя с ID: {}", event_id));
  app.unlisten(event_id);

  log(LogLevel::Info, "close_serial_port", format!("Остановка прослушивания порта: {}", path));
  let _ = stop_listening(app.clone(), serial.clone(), path.clone()).map_err(|e| {
    log(
      LogLevel::Err,
      "close_serial_port",
      format!("Не удалось остановить прослушивание порта {}: {}", path.clone(), e),
    );
    e.to_string()
  });
  match close(app.clone(), serial.clone(), path.clone()) {
    Ok(_) => {
      log(LogLevel::Info, "close_serial_port", format!("Порт {} успешно закрыт", path));

      if let Err(e) = app
        .clone()
        .emit("app-status", format!("Port {} closed successfully", path.clone()))
      {
        eprintln!("Failed to emit data: {}", e);
      }
    },
    Err(e) => {
      log(LogLevel::Err, "close_serial_port", format!("Не удалось закрыть порт {}: {}", path.clone(), e));

      if let Err(e2) = app
        .clone()
        .emit("app-status", format!("Failed to close port {}: {}", path.clone(), e))
      {
        eprintln!("Failed to emit data: {}", e2);
      }

      log(LogLevel::Info, "close_serial_port", format!("Попытка принудительного закрытия порта: {}", path));
      if let Err(e3) = force_close(app.clone(), serial.clone(), path.clone()) {
        log(
          LogLevel::Err,
          "close_serial_port",
          format!("Не удалось принудительно закрыть порт {}: {}", path.clone(), e3),
        );

        if let Err(e4) = app
          .clone()
          .emit("app-status", format!("Failed to force close port {}: {}", path.clone(), e3))
        {
          eprintln!("Failed to emit data: {}", e4);
        }
      }
    },
  }

  log(LogLevel::Info, "close_serial_port", format!("Процесс закрытия порта завершён"));

  Ok(())
}

/// Отправляет данные в серийный порт по указанному протоколу.
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `serial` - состояние серийного порта
/// * `protocol` - протокол передачи данных (SimpleSerial, POESerial, POECanable, POECanableFD)
/// * `port_path` - путь к порту для отправки данных
/// * `command_data` - JSON-объект с данными команды
///
/// # Returns
/// * `Ok(())` - данные успешно отправлены
/// * `Err(String)` - ошибка отправки
#[command]
pub async fn process_data_sending(
  app: AppHandle,
  serial: State<'_, SerialPort<Wry>>,
  protocol: String,
  port_path: String,
  command_data: serde_json::Value,
) -> Result<(), String> {
  log(
    LogLevel::Info,
    "process_data_sending",
    format!("Начало отправки данных по протоколу {} на порт {}", protocol, port_path),
  );

  // Сопоставляем протокол с соответствующей функцией отправки
  match protocol.clone().as_str() {
    "SimpleSerial" => {
      log(LogLevel::Info, "process_data_sending", format!("Отправка команды по протоколу SimpleSerial"));

      // Вызываем функцию отправки для SimpleSerial
      match send_simple_serial_command(app.clone(), serial.clone(), port_path.clone(), command_data) {
        Ok(_) => {
          log(LogLevel::Info, "process_data_sending", format!("Команда SimpleSerial успешно отправлена"));

          if let Err(e) = app
            .clone()
            .emit("app-status", format!("Data was successfully sent"))
          {
            eprintln!("Failed to emit data: {}", e);
          }
        },
        Err(e) => {
          log(LogLevel::Err, "process_data_sending", format!("Ошибка отправки команды SimpleSerial: {}", e));

          if let Err(e2) = app
            .clone()
            .emit("app-status", format!("Failed to send data: {}", e))
          {
            eprintln!("Failed to emit data: {}", e2);
          }
        },
      }
    },
    "POESerial" => {
      log(LogLevel::Info, "process_data_sending", format!("Отправка команды по протоколу POESerial"));

      // Логируем начало отправки по POESerial протоколу
      match send_poe_serial_command(app.clone(), serial.clone(), port_path.clone(), command_data) {
        Ok(_) => {
          log(LogLevel::Info, "process_data_sending", format!("Команда POESerial успешно отправлена"));

          if let Err(e) = app
            .clone()
            .emit("app-status", format!("Data was successfully sent"))
          {
            eprintln!("Failed to emit data: {}", e);
          }
        },
        Err(e) => {
          if let Err(e2) = app
            .clone()
            .emit("app-status", format!("Failed to send data: {}", e))
          {
            eprintln!("Failed to emit data: {}", e2);
          }
        },
      }
    },
    "POECanable" | "POECanableFD" => {
      log(LogLevel::Info, "process_data_sending", format!("Отправка команды по протоколу {}", protocol));

      // Вызываем функцию отправки для POECanable
      match send_poe_canable_command(app.clone(), serial.clone(), protocol.clone(), port_path.clone(), command_data) {
        Ok(_) => {
          log(LogLevel::Info, "process_data_sending", format!("Команда {} успешно отправлена", protocol));

          if let Err(e) = app
            .clone()
            .emit("app-status", format!("Data was successfully sent"))
          {
            eprintln!("Failed to emit data: {}", e);
          }
        },
        Err(e) => {
          log(LogLevel::Err, "process_data_sending", format!("Ошибка отправки команды {}: {}", protocol, e));

          if let Err(e2) = app
            .clone()
            .emit("app-status", format!("Failed to send data: {}", e))
          {
            eprintln!("Failed to emit data: {}", e2);
          }
        },
      }
    },
    _ => {
      log(LogLevel::Warn, "process_data_sending", format!("Неизвестный протокол: {}", protocol));
    },
  };

  log(LogLevel::Info, "process_data_sending", format!("Процесс отправки данных завершён"));

  Ok(())
}

/// Выполняет жёсткий перезапуск устройства через установку DTR/RTS флагов.
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `serial` - состояние серийного порта
/// * `path` - путь к порту для перезапуска
///
/// # Returns
/// * `Ok(())` - перезапуск успешно выполнен
/// * `Err(())` - ошибка выполнения
#[command]
pub async fn hard_restart(app: AppHandle<Wry>, serial: State<'_, SerialPort<Wry>>, path: String) -> Result<(), ()> {
  log(
    LogLevel::Info,
    "hard_restart",
    format!("Начало жёсткого перезапуска устройства на порту: {}", path),
  );

  // Шаг 1: Устанавливаем DTR в true и RTS в false
  log(LogLevel::Info, "hard_restart", format!("Установка DTR в true и RTS в false"));
  let _ = write_data_terminal_ready(app.clone(), serial.clone(), path.to_string().clone(), true).map_err(|e| e.to_string());
  let _ = write_request_to_send(app.clone(), serial.clone(), path.to_string().clone(), false).map_err(|e| e.to_string());

  tokio::time::sleep(Duration::from_millis(100)).await;

  // Шаг 2: Устанавливаем DTR в false и RTS в true
  log(LogLevel::Info, "hard_restart", format!("Установка DTR в false и RTS в true"));
  let _ = write_data_terminal_ready(app.clone(), serial.clone(), path.to_string().clone(), false).map_err(|e| e.to_string());
  let _ = write_request_to_send(app.clone(), serial.clone(), path.to_string().clone(), true).map_err(|e| e.to_string());

  tokio::time::sleep(Duration::from_millis(100)).await;

  // Шаг 3: Устанавливаем DTR и RTS в true (завершение последовательности)
  log(LogLevel::Info, "hard_restart", format!("Установка DTR и RTS в true"));
  let _ = write_data_terminal_ready(app.clone(), serial.clone(), path.to_string().clone(), true).map_err(|e| e.to_string());
  let _ = write_request_to_send(app.clone(), serial.clone(), path.to_string().clone(), true).map_err(|e| e.to_string());

  Ok(())
}
