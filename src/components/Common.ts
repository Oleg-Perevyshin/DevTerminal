import { APP_LOG_LEVEL } from "../appConfig"

export const log = (level: "ERR" | "WARN" | "INFO", label: string, ...args: unknown[]) => {
  if (APP_LOG_LEVEL === "none") {
    return
  } else if (APP_LOG_LEVEL === "error" && level === "ERR") {
    console.log(`[${new Date().toLocaleTimeString()}] ${level}: ${label} |`, ...args)
  } else if (APP_LOG_LEVEL === "warning" && (level === "ERR" || level === "WARN")) {
    console.log(`[${new Date().toLocaleTimeString()}] ${level}: ${label} |`, ...args)
  } else if (APP_LOG_LEVEL === "info" && (level === "ERR" || level === "WARN" || level === "INFO")) {
    console.log(`[${new Date().toLocaleTimeString()}] ${level}: ${label} |`, ...args)
  } else if (APP_LOG_LEVEL === "debug") {
    console.log(`[${new Date().toLocaleTimeString()}] ${level}: ${label} |`, ...args)
  }
}
