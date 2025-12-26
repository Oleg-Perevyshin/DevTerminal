import type { ITableHeader } from "poe-svelte-ui-lib"
import { ARGUMENT_MAP, HEADER_MAP } from "../stores/Dictionary"

/* Интерфейс для отправки данных */
export interface POECanableData {
  main_id?: number
  port?: string
  timestamp?: number
  time_delta?: string
  length?: string
  header: string | number
  argument: string | number
  target_id: string | number
  return_id: string | number
  convert_to_base64?: number
  data?: string
}

/* Интерфейсы для получения данных */
export interface FullId {
  isFullPacket: number
  header_code: number
  argument_code: number
  target_id: number
  return_id: number
}

export interface MessageData {
  timestamp: number
  full_id: FullId
  mainID: number
  can_data: Uint8Array
  json: string
  is_remote: boolean
  is_complete: boolean
}

export interface IPOECanableTableRow {
  time_delta: string
  length: string
  header: string
  argument: string
  targetID: string
  returnID: string
  data: string
}

export const POECanableTableColumns: ITableHeader<IPOECanableTableRow>[] = [
  {
    label: { name: "Δt" },
    key: "time_delta",
    width: "4rem",
    align: "center",
  },
  {
    label: { name: "Len" },
    key: "length",
    width: "3rem",
    align: "center",
  },
  {
    label: { name: "H" },
    key: "header",
    width: "1.5rem",
    align: "center",
    overflow: {
      truncated: true,
      formatting: text => {
        for (const [key, value] of Object.entries(ARGUMENT_MAP)) {
          if (value === Number(text)) {
            return key
          }
        }
        return text
      },
    },
  },
  {
    label: { name: "Arg" },
    key: "argument",
    width: "3rem",
    align: "left",
    overflow: {
      truncated: true,
      formatting: text => {
        for (const [key, value] of Object.entries(ARGUMENT_MAP)) {
          if (value === Number(text.replace(/^0+/, ""))) {
            return key
          }
        }
        return text
      },
    },
  },
  {
    label: { name: "T" },
    key: "targetID",
    width: "3rem",
    align: "left",
  },
  {
    label: { name: "R" },
    key: "returnID",
    width: "3rem",
    align: "left",
  },
  {
    label: { name: "Data" },
    key: "data",
    width: "1fr",
    align: "left",
    overflow: {
      truncated: true,
      copy: true,
      modal: true,
    },
  },
]
