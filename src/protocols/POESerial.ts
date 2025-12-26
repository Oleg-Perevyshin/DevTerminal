import type { ITableHeader } from "poe-svelte-ui-lib"

/* Интерфейс для отправки данных */
export interface POESerialData {
  header: string
  argument: string
  value: string
  crc_hex?: string
  free_heap_size?: string
}

export interface IPOESerialTableRow {
  header: string
  argument: string
  value: string
  crc_hex: string
  free_heap_size: string
}

export const POESerialTableColumns: ITableHeader<IPOESerialTableRow>[] = [
  {
    label: { name: "Header" },
    key: "header",
    width: "3.5rem",
    align: "left",
  },
  {
    label: { name: "Argument" },
    key: "argument",
    width: "8.5rem",
    align: "left",
  },
  {
    label: { name: "Value" },
    key: "value",
    width: "1fr",
    align: "left",
    overflow: {
      truncated: true,
      copy: true,
      formatting: text => {
        try {
          const jsonValue = JSON.parse(text ? text : "")
          return formatJson(jsonValue)
        } catch {
          return text
        }
      },
      modal: true,
    },
  },
  {
    label: { name: "CRC" },
    key: "crc_hex",
    width: "3rem",
    align: "left",
  },
  {
    label: { name: "FHS" },
    key: "free_heap_size",
    width: "5.5rem",
    align: "left",
  },
]

/* Форматирование JSON */
const formatJson = (value: object, indent: number = 2): string => {
  if (!Array.isArray(value)) {
    const indentation = " ".repeat(indent)
    if (typeof value === "object" && value !== null) {
      return (
        "{<br>" +
        Object.entries(value)
          .map(([key, val]) => {
            return `${indentation}<span class="text-left text-fuchsia-500">${JSON.stringify(key)}:</span> ${formatJson(val, indent + 2)}`
          })
          .join(",<br>") +
        `<br>${" ".repeat(indent - 2)}}`
      )
    } else if (typeof value === "string") {
      return `<span class="text-left text-violet-700">${JSON.stringify(value)}</span>`
    } else if (typeof value === "number") {
      return `<span class="text-left text-red-700">${value}</span>`
    } else if (typeof value === "boolean") {
      return `<span class="text-left text-pink-600">${value}</span>`
    } else {
      return `<span class="text-left text-teal-600">null</span>`
    }
  } else {
    const indentation = " ".repeat(indent)
    return (
      "[<br>" +
      value
        .map(item => {
          return `${indentation}${formatJson(item, indent + 2)}`
        })
        .join(",<br>") +
      `<br>${" ".repeat(indent - 2)}]`
    )
  }
}
