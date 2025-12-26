use tauri_plugin_serialplugin::state::{DataBits, FlowControl, Parity, StopBits};

/// Конвертирует числовое значение в enum DataBits.
/// 
/// # Arguments
/// * `value` - числовое значение (5, 6, 7, 8)
/// 
/// # Returns
/// * `Some(DataBits)` - соответствующее значение enum
/// * `None` - если значение не поддерживается
pub fn data_bits_from_u32(value: u32) -> Option<DataBits> {
  match value {
    5 => Some(DataBits::Five),
    6 => Some(DataBits::Six),
    7 => Some(DataBits::Seven),
    8 => Some(DataBits::Eight),
    _ => None,
  }
}

/// Конвертирует числовое значение в enum FlowControl.
/// 
/// # Arguments
/// * `value` - числовое значение (0, 1, 2)
/// 
/// # Returns
/// * `Some(FlowControl)` - соответствующее значение enum
/// * `None` - если значение не поддерживается
pub fn flow_control_from_u32(value: u32) -> Option<FlowControl> {
  match value {
    0 => Some(FlowControl::None),
    1 => Some(FlowControl::Software),
    2 => Some(FlowControl::Hardware),
    _ => None,
  }
}

/// Конвертирует числовое значение в enum Parity.
/// 
/// # Arguments
/// * `value` - числовое значение (0, 1, 2)
/// 
/// # Returns
/// * `Some(Parity)` - соответствующее значение enum
/// * `None` - если значение не поддерживается
pub fn parity_from_u32(value: u32) -> Option<Parity> {
  match value {
    0 => Some(Parity::None),
    1 => Some(Parity::Odd),
    2 => Some(Parity::Even),
    _ => None,
  }
}

/// Конвертирует числовое значение в enum StopBits.
/// 
/// # Arguments
/// * `value` - числовое значение (1, 2)
/// 
/// # Returns
/// * `Some(StopBits)` - соответствующее значение enum
/// * `None` - если значение не поддерживается
pub fn stop_bits_from_u32(value: u32) -> Option<StopBits> {
  match value {
    1 => Some(StopBits::One),
    2 => Some(StopBits::Two),
    _ => None,
  }
}