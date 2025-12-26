/* src-tauri\src\models.rs */

use serde::{Deserialize, Serialize};

/* Структура для параметров подключения */
#[derive(Debug, Deserialize, Serialize)]
pub struct SerialConfig {
  pub path: String,
  pub baud_rate: u32,
  pub data_bits: u32,
  pub flow_control: u32,
  pub parity: u32,
  pub stop_bits: u32,
  pub timeout: Option<u64>,
  pub protocol: String,
  pub can_bitrate: Option<String>,
  pub canfd_bitrate: Option<String>,
  pub canfd_data_bitrate: Option<String>,
}
/* Структуры для передачи данных на фронтенд */
#[derive(Serialize, Clone)]
pub struct SerialDataPayload {
  pub port: String,
  pub data: String,
}

#[derive(Serialize, Clone)]
pub struct SerialErrorPayload {
  pub port: String,
  pub error: String,
}

/* Структура для приема данных из библиотеки */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadDataResult {
  pub size: usize,
  pub data: Vec<u8>,
}
