use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tauri::{command, AppHandle, Listener, State, Wry};
use tauri_plugin_serialplugin::commands::write;
use tauri_plugin_serialplugin::desktop_api::SerialPort;

use crate::{log, LogLevel, ReadDataResult};

/* [SOH] HEADER [US] ARGUMENT [STX] VALUE [ETX] CRC8 [EOT] */

/// Структура данных для протокола POESerial
#[derive(serde::Serialize, Clone)]
pub struct PoeSerialData {
  header: String,
  argument: String,
  value: String,
  crc_hex: String,
  free_heap_size: String,
}

/// Структура команды для протокола POESerial
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct PoeSerialCommand {
  header: String,
  argument: String,
  value: String,
}

/// Константы управляющих символов для протокола POESerial
const SOH: u8 = 0x01;
const STX: u8 = 0x02;
const ETX: u8 = 0x03;
const EOT: u8 = 0x04;
const US: u8 = 0x1F;

/// Структура для хранения частичного пакета с временной меткой
#[derive(Debug)]
struct PartialPacket {
  data: String,
  timestamp: Instant,
}

lazy_static! {
  static ref DATA_BUFFERS: Arc<Mutex<HashMap<String, Vec<PoeSerialData>>>> = Arc::new(Mutex::new(HashMap::new()));
  static ref PARTIAL_PACKETS: Arc<Mutex<HashMap<String, PartialPacket>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Обрабатывает принятые данные по протоколу POESerial и отправляет их через канал
///
/// # Arguments
/// * `app` - дескриптор приложения Tauri
/// * `port_path` - путь к серийному порту
/// * `on_event` - канал для отправки обработанных данных (Vec<PoeSerialData>)
///
/// # Returns
/// * `Ok(u32)` - ID события прослушивания
/// * `Err(String)` - ошибка при создании слушателя
#[command]
pub fn process_poe_serial(app: AppHandle<Wry>, port_path: String, on_event: Channel<Vec<PoeSerialData>>) -> Result<u32, String> {
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
    "process_poe_serial",
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
      log(LogLevel::Info, "process_poe_serial", format!("Данные в виде строки: {}", data_str));

      // Добавляем данные в буфер
      {
        let mut buffer_guard = buffer_clone.lock().unwrap();
        buffer_guard.push_str(&data_str);
        log(
          LogLevel::Info,
          "process_poe_serial",
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
        match process_poe_serial_data(&data, &on_event, &port_path) {
          Ok(remaining) => remaining,
          Err(e) => {
            log(LogLevel::Err, "process_poe_serial", format!("Ошибка обработки данных: {}", e));
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

/// Вспомогательная функция для обработки данных POESerial
///
/// # Arguments
/// * `data` - строка с данными для обработки
/// * `on_event` - канал для отправки обработанных данных
/// * `port_path` - путь к порту (для логирования и работы с частичными пакетами)
///
/// # Returns
/// * `Ok(String)` - оставшиеся необработанные данные
/// * `Err(String)` - ошибка обработки
fn process_poe_serial_data(data: &str, on_event: &Channel<Vec<PoeSerialData>>, port_path: &str) -> Result<String, String> {
  log(
    LogLevel::Info,
    "process_poe_serial_data",
    format!("Начало обработки POESerial данных для порта: {}", port_path),
  );

  let current_time = Instant::now();

  // Очистка устаревших частичных пакетов (старше 0.5 секунды)
  {
    let mut partial_packets = PARTIAL_PACKETS.lock().unwrap();
    partial_packets.retain(|_, packet| current_time.duration_since(packet.timestamp) < Duration::from_millis(500));
  }

  let mut remaining_data = data.to_string();
  let mut partial_data = String::new();
  {
    let mut partial_packets = PARTIAL_PACKETS.lock().unwrap();
    if let Some(partial_packet) = partial_packets.remove(port_path) {
      if current_time.duration_since(partial_packet.timestamp) < Duration::from_millis(500) {
        partial_data = partial_packet.data;
        log(LogLevel::Info, "process_poe_serial_data", format!("Восстановлены частичные данные из буфера"));
      }
    }
  }

  partial_data.push_str(&remaining_data);
  remaining_data = partial_data;
  log(
    LogLevel::Info,
    "process_poe_serial_data",
    format!("Данные после объединения с частичными: {}", remaining_data),
  );

  let mut packets_to_send = Vec::new();

  // Обрабатываем пакеты в данных
  while remaining_data.as_bytes().contains(&SOH) && remaining_data.as_bytes().contains(&EOT) {
    // Поиск начала и конца пакета
    let eot_index = match remaining_data.find(EOT as char) {
      Some(index) => index,
      None => {
        log(LogLevel::Info, "process_poe_serial_data", format!("Не найден символ EOT, прерывание цикла"));
        break;
      },
    };
    let soh_index = match remaining_data.find(SOH as char) {
      Some(index) => index,
      None => {
        log(LogLevel::Info, "process_poe_serial_data", format!("Не найден символ SOH, прерывание цикла"));
        break;
      },
    };

    if soh_index >= eot_index {
      log(LogLevel::Warn, "process_poe_serial_data", format!("Нарушен порядок SOH и EOT, пропуск пакета"));

      break;
    }

    // Выделяем полный пакет
    let packet = remaining_data[soh_index..=eot_index].to_string();

    log(LogLevel::Info, "process_poe_serial_data", format!("Найден пакет: {}", packet));

    if !packet.as_bytes().contains(&US) || !packet.as_bytes().contains(&STX) || !packet.as_bytes().contains(&ETX) {
      log(
        LogLevel::Info,
        "process_poe_serial_data",
        format!("Пакет не содержит необходимые разделители, пропуск"),
      );
      remaining_data = remaining_data[eot_index + 1..].to_string();
      continue;
    }

    let trimmed_packet = packet[packet.find(ETX as char).unwrap()..].to_string();

    // Разделение пакета по элементам
    let header = packet[1..packet.find(US as char).unwrap()].to_string();
    let argument = packet[packet.find(US as char).unwrap() + 1 as usize..packet.find(STX as char).unwrap()].to_string();
    let value = packet[packet.find(STX as char).unwrap() + 1 as usize..packet.find(ETX as char).unwrap()].to_string();
    let crc_hex = trimmed_packet[trimmed_packet.find(ETX as char).unwrap() + 1 as usize..trimmed_packet.find(US as char).unwrap()].to_string();
    let free_heap_size = trimmed_packet[trimmed_packet.find(US as char).unwrap() + 1 as usize..trimmed_packet.find(EOT as char).unwrap()].to_string();

    // Создаём структуру данных пакета
    let serial_data = PoeSerialData {
      header,
      argument,
      value,
      crc_hex,
      free_heap_size,
    };

    log(
      LogLevel::Info,
      "process_poe_serial_data",
      format!(
        "Разобран пакет: header={}, argument={}, value={}",
        serial_data.header, serial_data.argument, serial_data.value
      ),
    );

    log(
      LogLevel::Info,
      "process_poe_serial_data",
      format!("Режим отправки: добавление пакета в очередь для отправки"),
    );

    packets_to_send.push(serial_data);

    // Удаляем обработанный пакет из remaining_data
    remaining_data = remaining_data[eot_index + 1..].to_string();
    log(
      LogLevel::Info,
      "process_poe_serial_data",
      format!("Оставшиеся данные после обработки пакета: {}", remaining_data),
    );
  }

  // Отправляем все накопленные пакеты через канал
  if !packets_to_send.is_empty() {
    log(
      LogLevel::Info,
      "process_poe_serial_data",
      format!("Отправка {} пакетов через канал", packets_to_send.len()),
    );
    on_event.send(packets_to_send).map_err(|e| e.to_string())?;
  }

  // Если остались неполные данные, сохраняем их с таймстампом
  if !remaining_data.is_empty() {
    log(
      LogLevel::Info,
      "process_poe_serial_data",
      format!("Сохранение частичных данных в буфере: {}", remaining_data),
    );

    let mut partial_packets = PARTIAL_PACKETS.lock().unwrap();
    partial_packets.insert(
      port_path.to_string(),
      PartialPacket {
        data: remaining_data.clone(),
        timestamp: current_time,
      },
    );
    remaining_data.clear();
  }

  log(LogLevel::Info, "process_poe_serial_data", format!("Обработка POESerial завершена"));
  Ok(remaining_data)
}

/// Отправляет команду по протоколу POESerial в серийный порт
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
pub fn send_poe_serial_command(app: AppHandle, serial: State<'_, SerialPort<Wry>>, port_path: String, sending_data: serde_json::Value) -> Result<(), String> {
  log(
    LogLevel::Info,
    "send_poe_serial_command",
    format!("Начало отправки POESerial команды на порт: {}", port_path),
  );

  // Разбираем JSON в структуру команды
  let command: PoeSerialCommand = serde_json::from_value(sending_data).map_err(|e| {
    log(
      LogLevel::Err,
      "send_poe_serial_command",
      format!("Не удалось разобрать команду POESerial: {}", e),
    );
    format!("Failed to parse simple serial command: {}", e)
  })?;

  log(
    LogLevel::Info,
    "send_poe_serial_command",
    format!(
      "Разобранные данные команды: header={}, argument={}, value={}",
      command.header, command.argument, command.value
    ),
  );

  let mut crc: u8 = 0x00;
  let data_to_checksum = format!("{}{}{}", command.header, command.argument, command.value);

  // Расчет CRC
  for ch in data_to_checksum.chars() {
    let byte = ch as u8;
    let mut extract = byte;

    for _ in 0..8 {
      let sum = (crc ^ extract) & 0x01;
      crc >>= 1;
      if sum != 0 {
        crc ^= 0x8C;
      }
      extract >>= 1;
    }
  }

  log(LogLevel::Info, "send_poe_serial_command", format!("Рассчитанное CRC: 0x{:02X}", crc));

  // Формируем посылку
  let formatted_str = format!(
    "{}{}{}{}{}{}{}{:02X}{}",
    SOH as char, command.header, US as char, command.argument, STX as char, command.value, ETX as char, crc, EOT as char
  );

  log(
    LogLevel::Info,
    "send_poe_serial_command",
    format!("Сформированная строка для отправки: {}", formatted_str),
  );

  write(app.clone(), serial.clone(), port_path.clone(), formatted_str.clone()).map_err(|e| {
    log(LogLevel::Err, "send_poe_serial_command", format!("Не удалось записать данные в порт: {}", e));
    format!("Failed to write: {}", e)
  })?;

  log(LogLevel::Info, "send_poe_serial_command", format!("Команда POESerial успешно отправлена"));
  Ok(())
}
