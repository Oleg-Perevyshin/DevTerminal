/* src\stores\StatusStore.ts */
import { writable } from "svelte/store"

export const StatusStore = writable("")
let timeoutId: ReturnType<typeof setTimeout> | null = null

export function UpdateStatus(message: string, duration: number = 5000) {
  StatusStore.set(message)

  if (timeoutId) {
    clearTimeout(timeoutId)
  }

  timeoutId = setTimeout(() => {
    StatusStore.set("") // Очищаем сообщение в store
    timeoutId = null
  }, duration)
}
