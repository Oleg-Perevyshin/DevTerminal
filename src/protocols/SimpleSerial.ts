import type { ITableHeader } from "poe-svelte-ui-lib"

/* Интерфейс для отправки данных */
export interface SimpleSerialData {
  data: string
  end_package?: string
}

export const SimpleSerialTableColumns: ITableHeader<SimpleSerialData>[] = [
  {
    label: { name: "Data" },
    key: "data",
    width: "1fr",
    sortable: false,
    align: "left",
    overflow: {
      truncated: true,
      copy: true,
      formatting: text => {
        try {
          return formatControlChars(text)
        } catch {
          return text
        }
      },
    },
  },
]

const formatControlChars = (data: string): string => {
  let formattedData: string = ""
  for (let i = 0; i < data.length; i++) {
    const char = data[i]
    const code = char.charCodeAt(0)
    if (code < 32 || code > 126) {
      formattedData += `<span class="text-red-300">[0x${code.toString(16).padStart(2, "0").toUpperCase()}]</span>`
    } else {
      formattedData += char
    }
  }
  return formattedData
}
