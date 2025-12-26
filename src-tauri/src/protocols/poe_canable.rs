use crate::{log, LogLevel, ReadDataResult};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::{command, AppHandle, Emitter, Listener, State, Wry};
use tauri_plugin_serialplugin::commands::write;
use tauri_plugin_serialplugin::desktop_api::SerialPort;

/// Структура для хранения расширенного ID CAN-фрейма
#[derive(serde::Serialize, Clone, Debug)]
pub struct FullId {
  pub is_full_packet: u32,
  pub header_code: u32,
  pub argument_code: u32,
  pub target_id: u32,
  pub return_id: u32,
}

/// Структура данных для сообщений POECanable
#[derive(serde::Serialize, Clone, Debug)]
pub struct MessageData {
  pub timestamp: u64,
  pub full_id: FullId,
  pub main_id: u32,
  #[serde(with = "serde_bytes")]
  pub can_data: Vec<u8>,
  pub json: String,
  pub is_remote: bool,
  pub is_complete: bool,
}

/// Структура для хранения частичного пакета
#[derive(Clone, Debug)]
pub struct PartialPacket {
  pub timestamp: u64,
  pub data: Vec<u8>,
}

/// Структура команды для протокола POECanable
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct PoeCANableCommand {
  header: u32,
  argument: u32,
  target_id: u32,
  return_id: u32,
  convert_to_base64: u32,
  data: Option<String>,
}

/// Тип для хранения сообщений (ID -> MessageData)
type MessagesMap = HashMap<u32, MessageData>;

/// Тип для хранения частичных пакетов по портам и ID
type PortPartialPackets = HashMap<String, HashMap<u32, PartialPacket>>;

lazy_static! {
    #[derive(Debug)]
    pub static ref PARTIAL_PACKETS: Arc<Mutex<PortPartialPackets>> =
        Arc::new(Mutex::new(HashMap::new()));
}

lazy_static! {
    #[derive(Debug)]
    static ref DATA_BUFFERS: Arc<Mutex<HashMap<String, Vec<(u32, MessageData)>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Обрабатывает принятые данные по протоколу POECanable и отправляет их через канал
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `port_path` - путь к серийному порту
/// * `on_event` - канал для отправки обработанных данных (Vec<(u32, MessageData)>)
///
/// # Returns
/// * `Ok(u32)` - ID события прослушивания
/// * `Err(String)` - ошибка при создании слушателя
#[command]
pub fn process_poe_canable(app: AppHandle<Wry>, port_path: String, on_event: Channel<Vec<(u32, MessageData)>>) -> Result<u32, String> {
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
    "process_poe_canable",
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
      log(LogLevel::Info, "process_poe_canable", format!("Данные в виде строки: {}", data_str));

      // Добавляем данные в буфер
      {
        let mut buffer_guard = buffer_clone.lock().unwrap();
        buffer_guard.push_str(&data_str);
        log(
          LogLevel::Info,
          "process_poe_canable",
          format!("Буфер обновлён, текущий размер: {}", buffer_guard.len()),
        );
        drop(buffer_guard);
      }

      let processed_remaining = {
        let buffer_guard = buffer_clone.lock().unwrap();
        let data = buffer_guard.clone();
        drop(buffer_guard);

        match process_poe_canable_data(&data, &on_event, &port_path) {
          Ok(remaining) => remaining,
          Err(e) => {
            log(LogLevel::Err, "process_poe_canable", format!("Ошибка обработки данных: {}", e));
            String::new()
          },
        }
      };

      let mut buffer_guard = buffer_clone.lock().unwrap();
      *buffer_guard = processed_remaining;
    }
  });

  Ok(event_id)
}

/// Вспомогательная функция для обработки данных POECanable
///
/// # Arguments
/// * `data` - строка с данными для обработки
/// * `on_event` - канал для отправки обработанных данных
/// * `port_path` - путь к порту (для логирования и работы с частичными пакетами)
///
/// # Returns
/// * `Ok(String)` - оставшиеся необработанные данные
/// * `Err(String)` - ошибка обработки
fn process_poe_canable_data(data: &str, on_event: &Channel<Vec<(u32, MessageData)>>, port_path: &str) -> Result<String, String> {
  const PACKET_TIMEOUT: u64 = 2000;

  log(
    LogLevel::Info,
    "process_poe_canable_data",
    format!("Начало обработки POECanable данных для порта: {}", port_path),
  );

  let mut remaining_data = data.to_string();
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis() as u64;

  log(LogLevel::Info, "process_poe_canable_data", format!("Очистка устаревших частичных пакетов"));

  // Очистка устаревших частичных пакетов для конкретного порта
  clear_expired_partial_packets(port_path, now, PACKET_TIMEOUT);

  let mut new_messages: MessagesMap = HashMap::new();

  // Регулярное выражение для поиска CAN-фреймов
  let can_frame_regex = Regex::new(r"([tTrRdDbB])([0-9A-F]{3,8})([0-9A-F])([0-9A-F]*)\r").map_err(|e| {
    log(
      LogLevel::Err,
      "process_poe_canable_data",
      format!("Ошибка компиляции регулярного выражения: {}", e),
    );
    format!("Regex error: {}", e)
  })?;

  // Обрабатываем все совпадения в цикле
  let mut processed_data = String::new();
  let mut last_processed_pos = 0;

  log(
    LogLevel::Info,
    "process_poe_canable_data",
    format!("Поиск CAN фреймов в строке: {}", remaining_data),
  );
  for cap in can_frame_regex.captures_iter(&remaining_data) {
    let match_start = cap.get(0).unwrap().start();
    let match_end = cap.get(0).unwrap().end();

    // Добавляем все данные до текущего совпадения в обработанные
    processed_data.push_str(&remaining_data[last_processed_pos..match_start]);

    log(
      LogLevel::Info,
      "process_poe_canable_data",
      format!("Найдено совпадение: {}", cap.get(0).unwrap().as_str()),
    );
    match process_can_frame(&cap, now, port_path, &mut new_messages) {
      Ok(_) => {
        log(LogLevel::Info, "process_poe_canable_data", format!("CAN фрейм успешно обработан"));
      },
      Err(e) => {
        log(LogLevel::Err, "process_poe_canable_data", format!("Ошибка обработки фрейма: {}", e));
        eprintln!("Error processing frame: {}", e);
      },
    }

    last_processed_pos = match_end;
  }

  processed_data.push_str(&remaining_data[last_processed_pos..]);
  remaining_data = processed_data;

  // Обновляем хранилище сообщений для конкретного порта
  if !new_messages.is_empty() {
    log(
      LogLevel::Info,
      "process_poe_canable_data",
      format!("Обнаружено {} новых сообщений для отправки", new_messages.len()),
    );
    let mut messages_to_send = Vec::new();

    for (main_id, mut message_data) in new_messages.drain() {
      log(
        LogLevel::Info,
        "process_poe_canable_data",
        format!("Режим отправки: добавление сообщения {} в очередь для отправки", main_id),
      );

      // Проверяем и декодируем Base64 данные
      let base64_regex = regex::Regex::new(r"^([0-9a-zA-Z+/]{4})*(([0-9a-zA-Z+/]{2}==)|([0-9a-zA-Z+/]{3}=))?$").map_err(|e| format!("Invalid regex: {}", e))?;

      let can_data_str = String::from_utf8_lossy(&message_data.can_data);
      if base64_regex.is_match(&can_data_str) {
        if let Ok(decoded_bytes) = BASE64_STANDARD.decode(can_data_str.as_bytes()) {
          message_data.can_data = decoded_bytes;
          log(
            LogLevel::Info,
            "process_poe_canable_data",
            format!("Данные успешно декодированы из Base64 для сообщения {}", main_id),
          );
        } else {
          log(
            LogLevel::Warn,
            "process_poe_canable_data",
            format!("Не удалось декодировать Base64 для сообщения {}", main_id),
          );
        }
      }

      messages_to_send.push((main_id, message_data));
    }

    // Отправляем все накопленные сообщения через канал
    if !messages_to_send.is_empty() {
      log(
        LogLevel::Info,
        "process_poe_canable_data",
        format!("Отправка {} сообщений через канал", messages_to_send.len()),
      );
      on_event.send(messages_to_send).map_err(|e| e.to_string())?;
    }
  } else {
    log(LogLevel::Info, "process_poe_canable_data", format!("Новых сообщений не обнаружено"));
  }

  log(LogLevel::Info, "process_poe_canable_data", format!("Обработка POECanable завершена"));
  Ok(remaining_data)
}

/// Функция для обработки CAN-фрейма
///
/// # Arguments
/// * `cap` - захваченные группы регулярного выражения
/// * `now` - текущее время
/// * `port_path` - путь к порту
/// * `new_messages` - mutable reference для добавления новых сообщений
///
/// # Returns
/// * `Ok(())` - фрейм успешно обработан
/// * `Err(String)` - ошибка обработки
fn process_can_frame(cap: &regex::Captures, now: u64, port_path: &str, new_messages: &mut MessagesMap) -> Result<(), String> {
  log(LogLevel::Info, "process_can_frame", format!("Начало обработки CAN фрейма"));

  // Извлекаем части фрейма из регулярного выражения
  let frame_type = &cap[1];
  let can_id_hex = &cap[2];
  let dlc_hex = &cap[3];
  let hex_data = &cap[4];

  log(
    LogLevel::Info,
    "process_can_frame",
    format!("Тип фрейма: {}, ID: {}, DLC: {}, данные: {}", frame_type, can_id_hex, dlc_hex, hex_data),
  );

  // Разбираем DLC и CAN ID из HEX
  let dlc = u32::from_str_radix(dlc_hex, 16).map_err(|e| {
    log(LogLevel::Err, "process_can_frame", format!("Не удалось разобрать DLC: {}", e));
    format!("Failed to parse DLC: {}", e)
  })?;
  let can_id = u32::from_str_radix(can_id_hex, 16).map_err(|e| {
    log(LogLevel::Err, "process_can_frame", format!("Не удалось разобрать CAN ID: {}", e));
    format!("Failed to parse CAN ID: {}", e)
  })?;

  // Определяем типы фрейма
  let is_extended = matches!(frame_type, "B" | "D" | "R" | "T");
  let is_remote = matches!(frame_type, "r" | "R");
  let is_can = matches!(frame_type, "t" | "T");
  let is_canfd = matches!(frame_type, "b" | "B" | "d" | "D");

  log(
    LogLevel::Info,
    "process_can_frame",
    format!(
      "Флаги: is_extended={}, is_remote={}, is_can={}, is_canfd={}",
      is_extended, is_remote, is_can, is_canfd
    ),
  );

  // Рассчитываем количество байт данных в зависимости от типа фрейма
  let data_bytes = if is_canfd {
    let size = match dlc as usize {
      0..=8 => dlc as usize,
      9 => 12,
      10 => 16,
      11 => 20,
      12 => 24,
      13 => 32,
      14 => 48,
      15 => 64,
      _ => 0,
    };
    log(LogLevel::Info, "process_can_frame", format!("Рассчитанный размер данных для CANFD: {}", size));
    size
  } else {
    let size = std::cmp::min(dlc as usize, 8);
    log(LogLevel::Info, "process_can_frame", format!("Рассчитанный размер данных для CAN: {}", size));
    size
  };

  // Разбираем HEX-данные в байты
  let mut bytes = if !is_remote {
    let mut result = Vec::new();
    let chars: Vec<char> = hex_data.chars().collect();
    let mut i = 0;

    while i + 1 < chars.len() && result.len() < data_bytes {
      let pair = format!("{}{}", chars[i], chars[i + 1]);
      if let Ok(byte) = u8::from_str_radix(&pair, 16) {
        result.push(byte);
        // log(
        //     LogLevel::Info,
        //     "process_can_frame",
        //     format!("Добавлен байт: 0x{:02X}", byte),
        // );
      } else {
        log(LogLevel::Warn, "process_can_frame", format!("Неверный байт в данных: {}", pair));
      }
      i += 2;
    }
    result
  } else {
    log(LogLevel::Info, "process_can_frame", format!("Фрейм типа remote, данные отсутствуют"));

    Vec::new()
  };

  // Для CANFD удаляем концевые нули
  if is_canfd {
    log(LogLevel::Info, "process_can_frame", format!("Применение обрезки нулей для CANFD данных"));
    if let Some(last_non_zero) = bytes.iter().rposition(|&b| b != 0) {
      bytes.truncate(last_non_zero + 1);
    } else {
      bytes.clear();
    }
  }

  // Разбираем расширенный ID
  let decoded_id = if is_extended {
    log(LogLevel::Info, "process_can_frame", format!("Разбор расширенного ID"));
    FullId {
      is_full_packet: (can_id >> 28) & 0x01,
      header_code: (can_id >> 26) & 0x03,
      argument_code: (can_id >> 16) & 0x3ff,
      target_id: (can_id >> 8) & 0xff,
      return_id: can_id & 0xff,
    }
  } else {
    log(LogLevel::Info, "process_can_frame", format!("ID не расширенный"));
    FullId {
      is_full_packet: 0,
      header_code: 0,
      argument_code: 0,
      target_id: 0,
      return_id: 0,
    }
  };

  // Рассчитываем основной ID
  let main_id = if is_extended { (can_id >> 16) & 0xfff } else { can_id & 0x7ff };
  log(LogLevel::Info, "process_can_frame", format!("Рассчитанный main_id: {}", main_id));

  // Обрабатываем remote фрейм
  if is_remote {
    log(LogLevel::Info, "process_can_frame", format!("Создание remote сообщения с ID: {}", main_id));
    let message = MessageData {
      timestamp: now,
      full_id: decoded_id.clone(),
      main_id,
      can_data: Vec::new(),
      json: "{}".to_string(),
      is_remote: true,
      is_complete: true,
    };
    new_messages.insert(main_id, message);
  }

  // Обрабатываем CAN/CANFD фреймы
  if is_can || is_canfd {
    if decoded_id.is_full_packet == 1 {
      // Полный пакет: объединяем с частичными данными
      log(LogLevel::Info, "process_can_frame", format!("Обнаружен полный пакет, ID: {}", main_id));
      let partial_data = get_partial_packet(port_path, main_id)
        .map(|p| p.data.clone())
        .unwrap_or_default();

      let complete_data: Vec<u8> = [partial_data.as_slice(), bytes.as_slice()].concat();

      // Пытаемся разобрать данные как JSON
      let json_str = if (is_canfd && !complete_data.is_empty() && complete_data[0] == 0x7b) || (!is_canfd && !complete_data.is_empty()) {
        match String::from_utf8(complete_data.clone()) {
          Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(_) => {
              log(LogLevel::Info, "process_can_frame", format!("Данные успешно разобраны как JSON"));
              text
            },
            Err(_) => {
              log(
                LogLevel::Info,
                "process_can_frame",
                format!("Данные не являются валидным JSON, возврат пустого объекта"),
              );
              "{}".to_string()
            },
          },
          Err(_) => {
            log(
              LogLevel::Info,
              "process_can_frame",
              format!("Данные не являются валидной строкой UTF-8, возврат пустого объекта"),
            );
            "{}".to_string()
          },
        }
      } else {
        log(
          LogLevel::Info,
          "process_can_frame",
          format!("Данные не начинаются с '{{', возврат пустого объекта"),
        );
        "{}".to_string()
      };

      // Получаем временную метку из частичного пакета или используем текущую
      let timestamp = get_partial_packet(port_path, main_id)
        .map(|p| p.timestamp)
        .unwrap_or(now);

      let message = MessageData {
        timestamp,
        full_id: decoded_id,
        main_id,
        can_data: complete_data,
        json: json_str,
        is_remote: false,
        is_complete: true,
      };

      new_messages.insert(main_id, message);

      log(LogLevel::Info, "process_can_frame", format!("Полное сообщение с ID {} добавлено", main_id));

      // Удаляем частичный пакет
      remove_partial_packet(port_path, main_id);
      log(LogLevel::Info, "process_can_frame", format!("Частичный пакет с ID {} удалён", main_id));
    } else {
      // Частичный пакет: объединяем с существующими или сохраняем как новый
      log(LogLevel::Info, "process_can_frame", format!("Обнаружен частичный пакет, ID: {}", main_id));
      let existing = get_partial_packet(port_path, main_id);

      let new_data: Vec<u8> = match existing {
        Some(ref existing_packet) => {
          log(LogLevel::Info, "process_can_frame", format!("Объединение с существующими частичными данными"));
          let combined: Vec<u8> = [existing_packet.data.as_slice(), bytes.as_slice()].concat();
          combined
        },
        None => {
          log(
            LogLevel::Info,
            "process_can_frame",
            format!("Нет существующих частичных данных, используем текущие"),
          );
          bytes
        },
      };

      // Ограничиваем размер частичного пакета
      if new_data.len() > 1024 {
        log(
          LogLevel::Info,
          "process_can_frame",
          format!("Частичные данные для ID {} превышают 1024 байта, удаление", main_id),
        );
        remove_partial_packet(port_path, main_id);
      } else {
        let timestamp = existing.map(|e| e.timestamp).unwrap_or(now);
        let packet = PartialPacket {
          timestamp,
          data: new_data.clone(),
        };

        insert_partial_packet(port_path, main_id, packet);
        log(LogLevel::Info, "process_can_frame", format!("Частичный пакет с ID {} обновлён", main_id));
      }
    }
  }

  log(LogLevel::Info, "process_can_frame", format!("Обработка CAN фрейма завершена"));
  Ok(())
}

/// Вставляет частичный пакет в хранилище
fn insert_partial_packet(port_path: &str, main_id: u32, packet: PartialPacket) {
  log(
    LogLevel::Info,
    "insert_partial_packet",
    format!("Вставка частичного пакета для порта {} с ID: {}", port_path, main_id),
  );

  let mut packets = PARTIAL_PACKETS.lock().unwrap();

  packets
    .entry(port_path.to_string())
    .or_insert_with(HashMap::new)
    .insert(main_id, packet);

  log(LogLevel::Info, "insert_partial_packet", format!("Частичный пакет для ID {} вставлен", main_id));
}

/// Получает частичный пакет из хранилища
fn get_partial_packet(port_path: &str, main_id: u32) -> Option<PartialPacket> {
  log(
    LogLevel::Info,
    "get_partial_packet",
    format!("Получение частичного пакета для порта {} с ID: {}", port_path, main_id),
  );

  let packets = PARTIAL_PACKETS.lock().unwrap();
  let result = packets
    .get(port_path)
    .and_then(|port_packets| port_packets.get(&main_id))
    .cloned();

  if result.is_some() {
    log(LogLevel::Info, "get_partial_packet", format!("Частичный пакет для ID {} найден", main_id));
  } else {
    log(LogLevel::Info, "get_partial_packet", format!("Частичный пакет для ID {} не найден", main_id));
  }

  result
}

/// Удаляет частичный пакет из хранилища
fn remove_partial_packet(port_path: &str, main_id: u32) {
  log(
    LogLevel::Info,
    "remove_partial_packet",
    format!("Удаление частичного пакета для порта {} с ID: {}", port_path, main_id),
  );

  let mut packets = PARTIAL_PACKETS.lock().unwrap();
  if let Some(port_packets) = packets.get_mut(port_path) {
    if port_packets.remove(&main_id).is_some() {
      log(LogLevel::Info, "remove_partial_packet", format!("Частичный пакет для ID {} удалён", main_id));
    } else {
      log(
        LogLevel::Info,
        "remove_partial_packet",
        format!("Частичный пакет для ID {} не существовал", main_id),
      );
    }
  } else {
    log(
      LogLevel::Info,
      "remove_partial_packet",
      format!("Для порта {} нет частичных пакетов", port_path),
    );
  }
}

/// Очищает устаревшие частичные пакеты из хранилища
fn clear_expired_partial_packets(port_path: &str, now: u64, packet_timeout: u64) {
  log(
    LogLevel::Info,
    "clear_expired_partial_packets",
    format!(
      "Очистка устаревших частичных пакетов для порта {}, текущее время: {}, таймаут: {}",
      port_path, now, packet_timeout
    ),
  );

  let mut packets = PARTIAL_PACKETS.lock().unwrap();
  if let Some(port_packets) = packets.get_mut(port_path) {
    let initial_count = port_packets.len();
    port_packets.retain(|_main_id, packet| now - packet.timestamp <= packet_timeout);
    let removed_count = initial_count - port_packets.len();
    if removed_count > 0 {
      log(
        LogLevel::Info,
        "clear_expired_partial_packets",
        format!("Очищено {} устаревших частичных пакетов для порта {}", removed_count, port_path),
      );
    }
  } else {
    log(
      LogLevel::Info,
      "clear_expired_partial_packets",
      format!("Для порта {} нет частичных пакетов для очистки", port_path),
    );
  }
}

/// Отправляет команду по протоколу POECanable в серийный порт
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `serial` - состояние серийного порта
/// * `protocol` - протокол ("POECanable", "POECanableFD")
/// * `port_path` - путь к серийному порту
/// * `sending_data` - JSON-объект с командой
///
/// # Returns
/// * `Ok(())` - команда успешно отправлена
/// * `Err(String)` - ошибка отправки
pub fn send_poe_canable_command(
  app: AppHandle,
  serial: State<'_, SerialPort<Wry>>,
  protocol: String,
  port_path: String,
  sending_data: serde_json::Value,
) -> Result<(), String> {
  log(
    LogLevel::Info,
    "send_poe_canable_command",
    format!("Начало отправки POECanable команды по протоколу {} на порт: {}", protocol, port_path),
  );

  // Разбираем JSON в структуру команды
  let command: PoeCANableCommand = serde_json::from_value(sending_data).map_err(|e| {
    log(
      LogLevel::Err,
      "send_poe_canable_command",
      format!("Не удалось разобрать команду POECanable: {}", e),
    );
    format!("Failed to parse simple serial command: {}", e)
  })?;

  // Проверяем валидность параметров ID
  if command.header > 0x3 || command.argument > 0x3ff || command.target_id > 0xff || command.return_id > 0xff {
    log(LogLevel::Err, "send_poe_canable_command", format!("Неверные параметры ID команды"));
    return Err("Invalid ID parameters".to_string());
  }

  // Рассчитываем CAN ID
  let can_id =
    (((command.header & 0x03) << 26) | ((command.argument & 0x3ff) << 16) | ((command.target_id & 0xff) << 8) | (command.return_id & 0xff)) & 0x1fffffff;

  log(LogLevel::Info, "send_poe_canable_command", format!("Рассчитанный CAN ID: 0x{:08X}", can_id));

  // Обрабатываем данные команды
  if let Some(ref data_str) = command.data {
    if data_str.trim().is_empty() {
      log(LogLevel::Info, "send_poe_canable_command", format!("Данные пусты, отправка remote фрейма"));
      let frame_id = can_id | (1 << 28);
      let formatted_str = format_can_frame('R', frame_id, None, 0)?;
      log(
        LogLevel::Info,
        "send_poe_canable_command",
        format!("Сформирован remote фрейм: {}", formatted_str),
      );

      let _ = write(app.clone(), serial.clone(), port_path.clone(), formatted_str.clone());
    } else {
      if command.convert_to_base64 == 1 {
        let mut bytes: Vec<u8> = Vec::new();
        for hex_part in data_str.split_whitespace() {
          match u8::from_str_radix(hex_part, 16) {
            Ok(byte) => {
              bytes.push(byte);
              log(LogLevel::Info, "send_poe_canable_command", format!("Добавлен байт из HEX: 0x{:02X}", byte));
            },
            Err(_) => {
              log(LogLevel::Err, "send_poe_canable_command", format!("Ошибка разбора HEX: {}", hex_part));
              eprintln!("Error HEX {}", hex_part)
            },
          }
        }
        log(LogLevel::Info, "send_poe_canable_command", format!("Данные для отправки: {}", data_str));

        let base64_str = if bytes.is_empty() {
          log(
            LogLevel::Info,
            "send_poe_canable_command",
            format!("Данные не являются HEX, конвертируем в байты как строку"),
          );
          let mut temp = data_str.bytes().collect::<Vec<u8>>();
          temp.push(0);
          temp
        } else {
          let mut temp = (BASE64_STANDARD.encode(String::from_utf8_lossy(&bytes.to_vec()).to_string()))
            .bytes()
            .collect::<Vec<u8>>();
          temp.push(0);
          temp
        };

        let max_frame_size = if protocol == "POECanableFD" { 64 } else { 8 };
        log(
          LogLevel::Info,
          "send_poe_canable_command",
          format!("Максимальный размер фрейма: {}", max_frame_size),
        );

        // Отправляем данные по частям
        for (offset, chunk) in base64_str.chunks(max_frame_size).enumerate() {
          let is_final = offset * max_frame_size + chunk.len() >= base64_str.len();
          let frame_id = can_id | if is_final { 1 << 28 } else { 0 };

          let frame_type = if protocol == "POECanableFD" { 'B' } else { 'T' };
          log(
            LogLevel::Info,
            "send_poe_canable_command",
            format!("Отправка фрейма {}, финальный: {}, ID: 0x{:08X}", offset, is_final, frame_id),
          );

          let formatted_str = format_can_frame(frame_type, frame_id, Some(chunk.to_vec()), chunk.len() as u32)?;
          log(LogLevel::Info, "send_poe_canable_command", format!("Сформирован фрейм: {}", formatted_str));

          if !chunk.to_vec().is_empty() {
            log(LogLevel::Info, "send_poe_canable_command", format!("Отправка подтверждения отправки данных"));

            if let Err(e) = app.emit(
              &format!("poe-canable-sending-data-{}", port_path.clone()),
              String::from_utf8_lossy(&base64_str).to_string(),
            ) {
              log(LogLevel::Err, "send_poe_canable_command", format!("Ошибка отправки подтверждения: {}", e));

              eprintln!("Failed to emit data: {}", e);
            }
          }

          let _ = write(app.clone(), serial.clone(), port_path.clone(), formatted_str.clone());
        }
      } else {
        // Отправляем данные без конвертации
        let bytes = data_str.as_bytes();
        let max_frame_size = if protocol == "poe_canfd" { 64 } else { 8 };

        for (offset, chunk) in bytes.chunks(max_frame_size).enumerate() {
          let is_final = offset * max_frame_size + chunk.len() >= bytes.len();
          let frame_id = can_id | if is_final { 1 << 28 } else { 0 };

          let frame_type = if protocol == "poe_canfd" { 'B' } else { 'T' };
          let formatted_str = format_can_frame(frame_type, frame_id, Some(chunk.to_vec()), chunk.len() as u32)?;

          let _ = write(app.clone(), serial.clone(), port_path.clone(), formatted_str.clone());
        }
      }
    }
  } else {
    log(
      LogLevel::Info,
      "send_poe_canable_command",
      format!("Данные отсутствуют, отправка remote фрейма"),
    );

    let frame_id = can_id | (1 << 28);
    let formatted_str = format_can_frame('R', frame_id, None, 0)?;
    log(
      LogLevel::Info,
      "send_poe_canable_command",
      format!("Сформирован remote фрейм: {}", formatted_str),
    );

    let _ = write(app.clone(), serial.clone(), port_path.clone(), formatted_str.clone());
  }

  log(LogLevel::Info, "send_poe_canable_command", format!("Команда POECanable успешно отправлена"));

  Ok(())
}

/// Формирует строку CAN-фрейма по заданным параметрам
///
/// # Arguments
/// * `frame_type` - тип фрейма ('t', 'T', 'r', 'R', 'd', 'D', 'b', 'B')
/// * `id` - CAN ID
/// * `data` - опциональные данные
/// * `dlc` - длина данных
///
/// # Returns
/// * `Ok(String)` - сформированная строка фрейма
/// * `Err(String)` - ошибка форматирования
fn format_can_frame(frame_type: char, id: u32, data: Option<Vec<u8>>, dlc: u32) -> Result<String, String> {
  log(
    LogLevel::Info,
    "format_can_frame",
    format!("Форматирование CAN фрейма: тип {}, ID 0x{:08X}, DLC {}", frame_type, id, dlc),
  );

  // Определяем, является ли фрейм расширенным (B, D, R, T) или CANFD (B, D, b, d)
  let is_extended = matches!(frame_type, 'B' | 'D' | 'R' | 'T');
  let is_canfd = matches!(frame_type, 'B' | 'D' | 'b' | 'd');

  // Форматируем ID: 8 символов для расширенного, 3 для стандартного
  let id_str = if is_extended { format!("{:08X}", id) } else { format!("{:03X}", id) };

  // Определяем символ DLC и фактическую длину данных
  let (dlc_char, actual_length) = if is_canfd {
    match dlc {
      0..=8 => (dlc.to_string(), dlc),
      9..=12 => ("9".to_string(), 12),
      13..=16 => ("A".to_string(), 16),
      17..=20 => ("B".to_string(), 20),
      21..=24 => ("C".to_string(), 24),
      25..=32 => ("D".to_string(), 32),
      33..=48 => ("E".to_string(), 48),
      49..=64 => ("F".to_string(), 64),
      _ => ("8".to_string(), 8),
    }
  } else {
    let length = std::cmp::min(dlc, 8);
    (length.to_string(), length)
  };

  // Формируем строку данных в HEX-формате
  let data_str = if let Some(data_bytes) = data {
    if !matches!(frame_type, 'R' | 'r') {
      let mut padded_data = vec![0u8; actual_length as usize];
      let copy_length = std::cmp::min(data_bytes.len(), actual_length as usize);
      padded_data[..copy_length].copy_from_slice(&data_bytes[..copy_length]);

      let hex_str = padded_data
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<String>();
      log(LogLevel::Info, "format_can_frame", format!("Данные фрейма в HEX: {}", hex_str));
      hex_str
    } else {
      log(LogLevel::Info, "format_can_frame", format!("Фрейм без данных"));
      String::new()
    }
  } else {
    String::new()
  };

  // Формируем итоговую строку фрейма
  let result = format!("{}{}{}{}\r", frame_type, id_str, dlc_char, data_str);
  log(LogLevel::Info, "format_can_frame", format!("Сформированная строка фрейма: {}", result));

  Ok(result)
}
