export const commandTabs: Record<string, string> = {
  "Simple Serial": "SimpleSerial",
  "POE Serial": "POESerial",
  "POE Canable": "POECanable",
}

export const HEADER_MAP: Record<string, number> = {
  GET: 0,
  SET: 1,
  "OK!": 2,
  "ER!": 3,
}

export const ARGUMENT_MAP: Record<string, number> = {
  /* Служебные команды */
  Restart: 0,
  DefaultConfig: 1,
  CreateBackup: 2,
  UpgradeDevice: 3,
  UpgradeByBinFile: 4,
  DeleteFile: 5,

  /* Локальная сеть */
  CheckWebSocket: 10,
  APList: 11,
  DeviceList: 12,
  FSInfo: 13,

  /* Модули в рамках изделия */
  ModuleList: 20,
  ModuleCapture: 21,
  ModuleInfo: 22,
  ModuleMode: 23,
  ModuleConfig: 24,
  ModuleTelemetry: 25,
  "ModuleStream-1": 26,
  "ModuleStream-2": 27,
  "ModuleStream-3": 28,

  /* Обновление прошивки */
  UpdateStart: 50,
  UpdateProcess: 51,
  UpdateCompleted: 52,
  UpdateError: 53,
}
