<script lang="ts">
  import * as UI from "poe-svelte-ui-lib"
  import { DataBits, Parity, StopBits } from "tauri-plugin-serialplugin-api"
  import {
    CAN_BITRATE,
    CAN_FD_BITRATE,
    CAN_FD_DATA_BITRATE,
    END_PACKAGE,
    POE_CANABLE_ARGUMENT,
    POE_CANABLE_HEADER,
    POE_SERIAL_ARGUMENT,
    POE_SERIAL_HEADER,
    PORT_RROTOCOL,
    SERIAL_BAUD_RATE,
    SERIAL_DATA_BITS,
    SERIAL_PARITY,
    SERIAL_STOP_BITS,
  } from "./options"
  import { UpdateStatus } from "../stores/StatusStore"
  import { onDestroy, onMount } from "svelte"
  import { Channel, invoke } from "@tauri-apps/api/core"
  import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event"
  import { SimpleSerialTableColumns, type SimpleSerialData } from "../protocols/SimpleSerial"
  import { POESerialTableColumns, type POESerialData } from "../protocols/POESerial"
  import { POECanableTableColumns, type MessageData, type POECanableData } from "../protocols/POECanable"
  import type { ISelectOption, SavedCommands } from "../stores/Interfaces"
  import ShowGraph from "../appIcons/ShowGraph.svelte"
  import SendCommand from "../appIcons/SendCommand.svelte"
  import CommandList from "../appIcons/CommandList.svelte"
  import CommandListModal from "./ElementsUI/CommandListModal.svelte"
  import Collapse from "../appIcons/Collapse.svelte"
  import { Block } from "./UI"
  import HardRestart from "../appIcons/HardRestart.svelte"
  import { slide } from "svelte/transition"
  import type { T } from "node_modules/tailwindcss/dist/types-CJYAW1ql.mjs"
  import { log } from "./Common"
  import ButtonClear from "../appIcons/ButtonClear.svelte"

  let {
    portList,
    isCollapsed = $bindable(),
  }: {
    portList: ISelectOption[]
    isCollapsed?: boolean
  } = $props()

  /* CONNECTION */
  let currentPort: string = $state("")
  let connectedPort: string = $state("")
  let formattedPortName = $derived(connectedPort.replace(/[.\/\\]/g, "-"))
  let isConnected = $state(false)
  let selectedProtocol: string = $state("SimpleSerial")

  /* Параметры для Serial */
  let selectedBaudRate = $state(115200)
  let selectedDataBits = $state("8")
  let selectedFlowControl = $state(0)
  let selectedParity = $state("0")
  let selectedStopBits = $state("1")
  /* Параметры для CAN */
  let selectedCanBitrate = $state("S8")
  let selectedCanFDBitrate = $state("S012F0C")
  let selectedCanFDDataBitrate = $state("Y010B03")
  $effect(() => {
    if (selectedCanFDBitrate == "S012F0C") {
      selectedCanFDDataBitrate = "Y010B03"
    }
  })
  /* COMMAND CREATOR */
  let showCommandListModal: boolean = $state(false)
  const defaultCommands: SavedCommands[] = [
    {
      SimpleSerial: { data: "Restart", endPackage: "\n", isShown: true },
      POESerial: { header: "SET", argument: "Restart", value: "{}", isShown: true },
      POECanable: { header: "1", argument: "0", targetID: "255", returnID: "255", data: "", isShown: true },
      name: "Restart",
    },
    {
      SimpleSerial: { data: "DefaultConfig", endPackage: "\n", isShown: true },
      POESerial: { header: "SET", argument: "DefaultConfig", value: "{}", isShown: true },
      POECanable: { header: "1", argument: "1", targetID: "255", returnID: "255", data: "", isShown: true },
      name: "Reset Config",
    },
    {
      SimpleSerial: { data: "ModuleConfig", endPackage: "\n", isShown: true },
      POESerial: { header: "GET", argument: "ModuleConfig", value: "{}", isShown: true },
      POECanable: { header: "0", argument: "24", targetID: "255", returnID: "255", data: "", isShown: true },
      name: "Get Config",
    },
    {
      SimpleSerial: { data: "ModuleList", endPackage: "\n", isShown: true },
      POESerial: { header: "GET", argument: "ModuleList", value: "{}", isShown: true },
      POECanable: { header: "0", argument: "20", targetID: "255", returnID: "255", data: "", isShown: true },
      name: "ModuleList",
    },
  ]

  let savedCommands: SavedCommands[] = $state([
    // {
    //   POESerial: { header: 'SET', argument: 'Restart', value: '{}', isShown: true },
    //   name: 'Restart',
    // },
    // {
    //   SimpleSerial: { data: 'ModuleList', endPackage: '\n', isShown: true },
    //   POECanable: { header: '0', argument: '4', targetID: '255', returnID: '255', data: '6565646', isShown: true },
    //   name: 'ModuleList',
    // },
    // {
    //   SimpleSerial: { data: 'SystemStatus', endPackage: '\n', isShown: true },
    //   name: 'SystemStatus',
    // },
    // {
    //   POESerial: { header: 'GET', argument: 'Version', value: '{}', isShown: true },
    //   name: 'GetVersion',
    // },
    // {
    //   POECanable: { header: '1', argument: '1', targetID: '100', returnID: '100', data: '00000001', isShown: true },
    //   name: 'SystemReset',
    // },
    // {
    //   SimpleSerial: { data: 'ConfigSave', endPackage: '\n', isShown: true },
    //   POESerial: { header: 'SET', argument: 'SaveConfig', value: 'fvdxfvxdfdxf', isShown: true },
    //   name: 'SaveConfig',
    // },
    // {
    //   POECanable: { header: '2', argument: '5', targetID: '101', returnID: '101', data: '00000002', isShown: true },
    //   name: 'LoadConfig',
    // },
    // {
    //   POESerial: { header: 'SET', argument: 'IP', value: '192.168.1.100', isShown: true },
    //   name: 'SetIP',
    // },
    // {
    //   SimpleSerial: { data: 'DeviceOn', endPackage: '\n', isShown: true },
    //   POESerial: { header: 'SET', argument: 'Power', value: 'ON', isShown: true },
    //   name: 'PowerOn',
    // },
    // {
    //   POECanable: { header: '3', argument: '10', targetID: '102', returnID: '102', data: '00000003', isShown: true },
    //   name: 'PowerOff',
    // },
    // {
    //   SimpleSerial: { data: 'Diagnostics', endPackage: '\n', isShown: true },
    //   POECanable: { header: '3', argument: '15', targetID: '103', returnID: '103', data: '00000004', isShown: true },
    //   name: 'RunDiagnostics',
    // },
    // {
    //   POESerial: { header: 'SET', argument: 'Temperature', value: '{}', isShown: true },
    //   name: 'ReadTemperature',
    // },
    // {
    //   SimpleSerial: { data: 'LogData', endPackage: '\n', isShown: true },
    //   name: 'GetLogs',
    // },
    // {
    //   POECanable: { header: '2', argument: '20', targetID: '104', returnID: '104', data: '00000005', isShown: true },
    //   name: 'SecurityReset',
    // },
    // {
    //   SimpleSerial: { data: 'MonitorStart', endPackage: '\n', isShown: true },
    //   name: 'StartMonitoring',
    // },
    // {
    //   POESerial: { header: 'FILE', argument: 'Delete', value: 'temp.txt', isShown: true },
    //   name: 'DeleteFile',
    // },
    // {
    //   POECanable: { header: '1', argument: '25', targetID: '105', returnID: '105', data: '00000006', isShown: true },
    //   name: 'UpdateFirmware',
    // },
    // {
    //   SimpleSerial: { data: 'UserList', endPackage: '\n', isShown: true },
    //   POESerial: { header: 'USER', argument: 'List', value: '{}', isShown: true },
    //   name: 'ListUsers',
    // },
    // {
    //   POECanable: { header: '0', argument: '30', targetID: '106', returnID: '106', data: '00000007', isShown: true },
    //   name: 'CalibrateSensors',
    // },
    // {
    //   SimpleSerial: { data: 'ProcessList', endPackage: '\n', isShown: true },
    //   POECanable: { header: '3', argument: '35', targetID: '107', returnID: '107', data: '00000008', isShown: true },
    //   name: 'ListProcesses',
    // },
  ])

  /* Параметры для Simple Serial */
  let simpleSerialInputStr: string = $state("")
  let selectedEndPackage: string = $state("\n")

  /* Параметры для POE Serial */
  let selectedPOESerialHeader: string = $state("GET")
  let selectedPOESerialArgument: string = $state("Restart")
  let POESerialValue: string = $state("{}")

  /* Параметры для POE CANable */
  let selectedCANableHeader: string = $state("0")
  let selectedCANableArgument: string = $state("0")
  let CANableTargetID: string = $state("255")
  let CANableReturnID: string = $state("255")
  let convertToBase64 = $state(0)
  let CANableData: string = $state("44 44 88")

  /* DATA LOG */
  let unlistenDisconnecting: UnlistenFn, unlistenFormattedData: UnlistenFn

  let event_id = $state()
  let modalData = $state({ isOpen: false, rawData: "", formattedData: "" })

  let SimpleSerialMessage: SimpleSerialData = $state({ data: "" })
  let POESerialMessages: POESerialData[] = $state([{ header: "", argument: "", value: "", crc_hex: "", free_heap_size: "" }])
  let POECanableSendingData: POECanableData = $state({
    port: "",
    time_delta: "",
    header: "",
    argument: "",
    target_id: "",
    return_id: "",
    data: "",
  })
  let POECanableMessages: POECanableData[] = $state([])

  let showGraph: boolean = $state(false)
  let dataForGraph: UI.IGraphDataObject[] = $state([])
  let timestampForGraph: number = $state(0)

  let tableRows = $derived(
    (() => {
      switch (selectedProtocol) {
        case "SimpleSerial":
          return { data: SimpleSerialMessage.data }
        case "POESerial":
          return POESerialMessages.map(message => ({
            header: message.header,
            argument: message.argument,
            value: message.value,
            crc_hex: message.crc_hex,
            free_heap_size: message.free_heap_size,
          }))
        case "POECanable":
        case "POECanableFD":
          return POECanableMessages.map(message => ({
            time_delta: message.time_delta,
            length: message.length,
            header: message.header,
            argument: message.argument,
            returnID: message.return_id,
            targetID: message.target_id,
            data: message.data,
          }))
        default:
          return SimpleSerialTableColumns
      }
    })(),
  )

  let tableColumns = $derived(
    (() => {
      switch (selectedProtocol) {
        case "SimpleSerial":
          return SimpleSerialTableColumns
        case "POESerial":
          return POESerialTableColumns
        case "POECanable":
        case "POECanableFD":
          return POECanableTableColumns
        default:
          return SimpleSerialTableColumns
      }
    })(),
  )
  let tableRef: any = $state(null)

  // ------------------------
  // === FUNCTIONS ===
  // ------------------------

  /* Подключение порта */
  const connect = async () => {
    log("INFO", "connect", "Начало процесса подключения")
    clearBuffers()
    POECanableMessages = [POECanableSendingData]

    const config = {
      path: currentPort,
      baud_rate: selectedProtocol === "POESerial" || selectedProtocol === "SimpleSerial" ? selectedBaudRate : 115200,
      data_bits: selectedProtocol === "POESerial" || selectedProtocol === "SimpleSerial" ? parseInt(selectedDataBits) : 8,
      flow_control: selectedProtocol === "POESerial" || selectedProtocol === "SimpleSerial" ? selectedFlowControl : 0,
      parity: selectedProtocol === "POESerial" || selectedProtocol === "SimpleSerial" ? parseInt(selectedParity) : 0,
      stop_bits: selectedProtocol === "POESerial" || selectedProtocol === "SimpleSerial" ? parseInt(selectedStopBits) : 1,
      timeout: 1000,
      protocol: selectedProtocol,
      can_bitrate: selectedCanBitrate,
      canfd_bitrate: selectedCanFDBitrate,
      canfd_data_bitrate: selectedCanFDDataBitrate,
    }

    log("INFO", "connect", "Конфигурация подготовлена", config)

    try {
      log("INFO", "connect", "Вызов команды подключения к порту")
      connectedPort = await invoke("connect_serial_port", { config })
      if (connectedPort) {
        log("INFO", "connect", "Успешное подключение к порту:", connectedPort)
        isConnected = true
        unlistenDisconnecting = await listen<T>(`plugin-serialplugin-disconnected-${formattedPortName}`, () => {
          disconnect(true)
          log("INFO", "connect", "Получено событие отключения, вызывается функция отключения")
        })
      } else {
        log("WARN", "connect", "Порт вернул ложное значение после успешного вызова команды.")
      }
    } catch (error) {
      log("ERR", "connect", `Ошибка подключения: ${error}`)
      UpdateStatus(`Ошибка подключения: ${error}`)
    }

    switch (selectedProtocol) {
      case "SimpleSerial": {
        const onEvent = new Channel<string>()
        onEvent.onmessage = message => {
          SimpleSerialMessage = { data: message }
          log("INFO", "SimpleSerial", "SimpleSerialMessage обновлён данными:", message)
        }
        await invoke(`process_simple_serial`, { portPath: connectedPort, onEvent }).then(id => {
          event_id = id
          log("INFO", "connect", "Начало непрерывного чтения данных с ID события:", event_id)
        })
        break
      }

      case "POESerial": {
        const onEvent = new Channel<POESerialData[]>()
        onEvent.onmessage = messages => {
          POESerialMessages = messages
          log("INFO", "POESerial", "POESerialMessages обновлён сообщением:", messages)
          if (showGraph) {
            messages.forEach(message => {
              try {
                const jsonValue = JSON.parse(message.value ? message.value : "")
                if (!Array.isArray(jsonValue)) {
                  log("WARN", "POESerial", "Разобранные данные не являются массивом, обновление графика пропущено")
                  return
                }
                log("INFO", "POESerial", "Обработка данных для графика из значения сообщения")
                dataForGraph = []
                for (const item of jsonValue) {
                  if (item && typeof item === "object" && typeof item.name === "string" && typeof item.value === "number") {
                    dataForGraph.push({
                      name: item.name,
                      value: item.value,
                    })
                    timestampForGraph = Date.now()
                    log("INFO", "POESerial", `Добавлена точка графика: ${item.name} = ${item.value}`)
                  } else {
                    log("WARN", "POESerial", "Неверный элемент в разобранных данных, обновление графика пропущено")
                    return
                  }
                }
              } catch (err) {
                log("ERR", "POESerial", `Ошибка разбора значения сообщения для графика: ${err}`)
                return
              }
            })
          }
        }
        log("INFO", "connect", "Настройка слушателя для протокола POESerial")

        await invoke(`process_poe_serial`, { portPath: connectedPort, onEvent }).then(id => {
          event_id = id
          log("INFO", "connect", "Начало непрерывного чтения данных с ID события:", event_id)
        })
        break
      }

      case "POECanable":
      case "POECanableFD": {
        log("INFO", "connect", `Настройка слушателя для протокола ${selectedProtocol}`)
        const onEvent = new Channel<[number, MessageData][]>()
        onEvent.onmessage = messages => {
          messages.forEach(([id, message]) => {
            const existingIndex = POECanableMessages.findIndex(item => item.main_id === id)
            let oldTimestamp = null

            if (existingIndex !== -1) {
              oldTimestamp = POECanableMessages[existingIndex].timestamp as number
              log("INFO", "POECanable", `Обновление существующего сообщения с индексом ${existingIndex}, старая метка времени: ${oldTimestamp}`)
              POECanableMessages[existingIndex] = {
                main_id: id,
                timestamp: message.timestamp,
                time_delta: formatTimeDelta(oldTimestamp ? message.timestamp - oldTimestamp : 0) ?? "N/A",
                length: message.can_data?.length?.toString() ?? "0",
                header: message.full_id?.header_code?.toString(10).toUpperCase().padStart(1, "0") ?? "N/A",
                argument: message.full_id?.argument_code?.toString(10).toUpperCase().padStart(4, "0") ?? "N/A",
                target_id: message.full_id?.target_id?.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
                return_id: message.full_id?.return_id?.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
                data:
                  formatHexStringByTwoChars(message.can_data?.reduce((acc, byte) => acc + byte.toString(16).padStart(2, "0").toUpperCase(), "")) ?? "No data",
              }
            } else {
              log("INFO", "POECanable", `Добавление нового сообщения с ID: ${id}`)
              POECanableMessages.push({
                main_id: id,
                timestamp: message.timestamp,
                time_delta: formatTimeDelta(0) ?? "N/A",
                length: message.can_data?.length?.toString() ?? "0",
                header: message.full_id?.header_code?.toString(10).toUpperCase().padStart(1, "0") ?? "N/A",
                argument: message.full_id?.argument_code?.toString(10).toUpperCase().padStart(4, "0") ?? "N/A",
                target_id: message.full_id?.target_id?.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
                return_id: message.full_id?.return_id?.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
                data:
                  formatHexStringByTwoChars(message.can_data?.reduce((acc, byte) => acc + byte.toString(16).padStart(2, "0").toUpperCase(), "")) ?? "No data",
              })
            }
            POECanableMessages = Array.from(POECanableMessages).sort((a, b) => a.main_id! - b.main_id!)
            log("INFO", "POECanable", "POECanableMessages отсортированы по ID")
          })
        }

        await invoke(`process_poe_canable`, { portPath: connectedPort, onEvent }).then(id => {
          event_id = id
          log("INFO", "connect", "Начало непрерывного чтения данных с ID события:", event_id)
        })

        break
      }
      default:
        log("WARN", "connect", `Неподдерживаемый протокол: ${selectedProtocol}`)
        break
    }
    log("INFO", "connect", "Настройка подключения завершена.")
  }

  /* Отключение порта */
  const disconnect = async (physically?: boolean) => {
    log("INFO", "disconnect", "Начало процесса отключения")
    try {
      if (connectedPort === currentPort) {
        log("INFO", "disconnect", `Отключение порта: ${connectedPort}, физическое отключение: ${!!physically}`)
        isConnected = false

        log("INFO", "disconnect", `Вызов команды закрытия порта: ${connectedPort}, ID события: ${event_id}`)
        await invoke("close_serial_port", {
          path: connectedPort,
          eventId: event_id,
          canProtocol: selectedProtocol.startsWith("POECanable") && !physically ? true : false,
        })
        log("INFO", "disconnect", `Порт ${currentPort} успешно отключен`)
        UpdateStatus(`Port ${currentPort} was successfully disconnected`)
      } else {
        log("INFO", "disconnect", `Порт ${currentPort} не был активно подключен, отмена действий`)
      }

      log("INFO", "disconnect", "Сброс значения connectedPort")
      connectedPort = ""
    } catch (err) {
      log("ERR", "disconnect", `Ошибка отключения порта ${currentPort}: ${err}`)
      UpdateStatus(`Error disconnecting port ${currentPort}: ${err}`)
    }
  }

  /* Обработчик отправки данных */
  const processDataSending = async (command: SavedCommands) => {
    log("INFO", "processDataSending", `Начало отправки команды для протокола: ${selectedProtocol}`)
    if (selectedProtocol == "SimpleSerial" && command.SimpleSerial) {
      log("INFO", "processDataSending", "Подготовка данных для SimpleSerial")
      let send: SimpleSerialData = { data: command.SimpleSerial.data, end_package: command.SimpleSerial.endPackage }
      log("INFO", "processDataSending", "Отправка данных SimpleSerial:", send)
      SimpleSerialMessage = { data: `<span class="text-emerald-600">${send.data}</span>` }
      await invoke("process_data_sending", { protocol: selectedProtocol, portPath: connectedPort, commandData: send })
      log("INFO", "processDataSending", "Команда SimpleSerial отправлена")
    } else if (selectedProtocol == "POESerial" && command.POESerial) {
      log("INFO", "processDataSending", "Подготовка данных для POESerial")
      let send: POESerialData = {
        header: command.POESerial.header,
        argument: command.POESerial.argument,
        value: command.POESerial.value,
      }
      log("INFO", "processDataSending", "Отправка данных POESerial:", send)
      POESerialMessages = [
        {
          header: `<span class="text-emerald-600">${send.header}</span>`,
          argument: `<span class="text-emerald-600">${send.argument}</span>`,
          value: `<span class="text-emerald-600">${send.value}</span>`,
        },
      ]
      await invoke("process_data_sending", { protocol: selectedProtocol, portPath: connectedPort, commandData: send })
      log("INFO", "processDataSending", "Команда POESerial отправлена")
    } else if (selectedProtocol.startsWith("POECanable") && command.POECanable) {
      log("INFO", "processDataSending", "Подготовка данных для POECanable")
      let sentData: string = ""
      let send: POECanableData = {
        header: parseInt(command.POECanable.header),
        argument: parseInt(command.POECanable.argument),
        target_id: parseInt(command.POECanable.targetID),
        return_id: parseInt(command.POECanable.returnID),
        convert_to_base64: convertToBase64,
        data: command.POECanable.data,
      }
      log("INFO", "processDataSending", "Отправка данных POECanable:", send)

      if (convertToBase64) {
        unlistenFormattedData = await listen<string>(`poe-canable-sending-data-${formattedPortName}`, event => {
          sentData = event.payload
          log("INFO", "processDataSending", `Получены подтверждение отправки: ${sentData}`)

          POECanableSendingData = POECanableMessages[0] = {
            length: send.data?.length.toString() ?? "0",
            header: send.header.toString(10).toUpperCase().padStart(1, "0") ?? "N/A",
            argument: send.argument.toString(10).toUpperCase().padStart(3, "0") ?? "N/A",
            target_id: send.target_id.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
            return_id: send.return_id.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
            data: `${send.data} (sent: ${sentData})`,
          }
        })
      } else {
        POECanableSendingData = POECanableMessages[0] = {
          length: send.data?.length.toString() ?? "0",
          header: send.header.toString(10).toUpperCase().padStart(1, "0") ?? "N/A",
          argument: send.argument.toString(10).toUpperCase().padStart(3, "0") ?? "N/A",
          target_id: send.target_id.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
          return_id: send.return_id.toString(16).toUpperCase().padStart(2, "0") ?? "N/A",
          data: `${send.data}`,
        }
      }

      await invoke("process_data_sending", { protocol: selectedProtocol, portPath: connectedPort, commandData: send })
      log("INFO", "processDataSending", "Команда POECanable отправлена")
      if (unlistenFormattedData) {
        unlistenFormattedData()
        log("INFO", "processDataSending", "Отключение временного слушателя подтверждения отправки")
      }
    } else {
      log("WARN", "processDataSending", `Нет данных для отправки по протоколу ${selectedProtocol} или команда не поддерживает текущий протокол`)
    }
    log("INFO", "processDataSending", "Процесс отправки данных завершён")
  }

  /* Отправить единичную команду */
  const sendingCommand = () => {
    log("INFO", "sendingCommand", `Начало формирования команды для протокола: ${selectedProtocol}`)
    let newCommand: SavedCommands
    switch (selectedProtocol) {
      case "SimpleSerial":
        newCommand = { SimpleSerial: { data: simpleSerialInputStr, endPackage: selectedEndPackage, isShown: true }, name: simpleSerialInputStr }
        log("INFO", "sendingCommand", "Сформирована команда SimpleSerial:", newCommand)
        break
      case "POESerial":
        newCommand = {
          POESerial: {
            header: selectedPOESerialHeader,
            argument: selectedPOESerialArgument,
            value: POESerialValue,
            isShown: true,
          },
          name: POESerialValue,
        }
        log("INFO", "sendingCommand", "Сформирована команда POESerial:", newCommand)
        break
      case "POECanable":
      case "POECanableFD":
        newCommand = {
          POECanable: {
            header: selectedCANableHeader,
            argument: selectedCANableArgument,
            targetID: CANableTargetID,
            returnID: CANableReturnID,
            data: CANableData,
            isShown: true,
          },
          name: CANableData,
        }
        log("INFO", "sendingCommand", `Сформирована команда ${selectedProtocol}:`, newCommand)
        break
      default:
        log("WARN", "sendingCommand", `Неподдерживаемый протокол: ${selectedProtocol}, команда не будет сформирована`)
        return
    }
    log("INFO", "sendingCommand", "Передача сформированной команды в обработчик отправки")
    processDataSending(newCommand)
    log("INFO", "sendingCommand", "Формирование и отправка команды завершены")
  }

  /* Форматирование CAN посылки */
  const formatHexStringByTwoChars = (dataStr: string): string => {
    log("INFO", "formatHexStringByTwoChars", `Начало форматирования строки: "${dataStr}"`)

    const cleanStr = dataStr.replace(/\s+/g, "").toUpperCase()
    log("INFO", "formatHexStringByTwoChars", `Строка после очистки: "${cleanStr}"`)

    if (cleanStr.length % 2 !== 0) {
      log("WARN", "formatHexStringByTwoChars", `Длина строки нечётная (${cleanStr.length}), возврат без форматирования`)
      return cleanStr
    }

    const byteGroups: string[] = []
    for (let i = 0; i < cleanStr.length; i += 2) {
      byteGroups.push(cleanStr.substr(i, 2))
      // log('INFO', 'formatHexStringByTwoChars', `Добавлен байт: ${cleanStr.substr(i, 2)}, индекс: ${i}`)
    }

    const result = byteGroups.join(" ")
    log("INFO", "formatHexStringByTwoChars", `Форматирование завершено, результат: "${result}"`)

    return result
  }

  /* Форматированиие времени между посылками */
  const formatTimeDelta = (ms: number): string => {
    log("INFO", "formatTimeDelta", `Форматирование временной разницы: ${ms} мс`)

    let result: string
    if (ms < 1) result = "0ms"
    else if (ms < 1000) result = `${ms}ms`
    else if (ms < 60000) result = `${(ms / 1000).toFixed(2)}s`
    else result = `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`

    log("INFO", "formatTimeDelta", `Форматирование завершено, результат: "${result}"`)
    return result
  }

  /* Очистка всех буферов */
  const clearBuffers = () => {
    tableRows = []
    tableRef.clearBuffer()

    POECanableSendingData = {
      port: "",
      time_delta: "",
      header: "",
      argument: "",
      target_id: "",
      return_id: "",
      data: "",
    }
    POECanableMessages = [POECanableSendingData]

    log("INFO", "clearBuffers", "Буферы очищены")
  }

  onMount(async () => {
    log("INFO", "onMount", "Монтирование компонента, настройка обработчиков")
    clearBuffers()

    log("INFO", "onMount", "Монтирование компонента завершено")
  })

  onDestroy(async () => {
    log("INFO", "onDestroy", "Начало размонтирования компонента")

    if (event_id) {
      await invoke("close_serial_port", {
        path: connectedPort,
        eventId: event_id,
        canProtocol: selectedProtocol.startsWith("POECanable") ? true : false,
      })
    }
    if (unlistenDisconnecting) unlistenDisconnecting()
    if (unlistenFormattedData) unlistenFormattedData()

    log("INFO", "onDestroy", "Размонтирование компонента завершено")
  })
</script>

<div class={`relative flex h-full flex-col ${isCollapsed ? "w-0 opacity-0" : "w-full opacity-100"} overflow-hidden duration-200 ease-in-out`}>
  <!-- CONNECTION -->
  <Block label="Connection" isAccordion isDropdownOpen={isConnected ? false : true} disabled={isConnected ? true : false}>
    <!-- Основная часть блока -->
    {#snippet main()}
      <UI.Button
        wrapperClass="absolute -top-2 right-5 bg-[#f5f7fa] h-6 w-8"
        content={{ info: { text: "Collapse", side: "left" }, icon: Collapse }}
        onClick={() => {
          isCollapsed = !isCollapsed
        }} />

      <div class="flex w-full items-end gap-2">
        <UI.Select
          wrapperClass="z-51"
          disabled={isConnected}
          label={{ name: "Serial Port" }}
          value={portList.find(p => p.value == currentPort)}
          options={portList}
          onUpdate={value => (currentPort = value.value as string)} />
        <UI.Button
          content={{ name: isConnected ? "Disconnect" : "Connect" }}
          wrapperClass="w-36"
          componentClass={`${isConnected ? "bg-red" : "bg-blue"} `}
          onClick={() => {
            isConnected ? disconnect() : connect()
          }} />
      </div>
    {/snippet}
    <!-- Дополнительная часть блока -->
    {#snippet details()}
      <div class="flex">
        <UI.Select
          wrapperClass="w-40"
          label={{ name: "Protocol" }}
          value={PORT_RROTOCOL.find(p => p.value === selectedProtocol)}
          options={PORT_RROTOCOL}
          onUpdate={value => {
            selectedProtocol = value.value as string
            clearBuffers()
          }} />
        <div class="flex grow items-center justify-center gap-2">
          <!-- Настройки для протокола SERIAL -->
          {#if selectedProtocol == "SimpleSerial" || selectedProtocol == "POESerial"}
            <UI.Select
              wrapperClass="w-28"
              label={{ name: "Baud Rate", class: "px-0" }}
              value={SERIAL_BAUD_RATE.find(a => a.value === selectedBaudRate)}
              options={SERIAL_BAUD_RATE}
              onUpdate={value => (selectedBaudRate = value.value as number)} />
            <UI.Select
              wrapperClass="w-28"
              label={{ name: "Data Bits", class: "px-0" }}
              value={SERIAL_DATA_BITS.find(a => a.value === selectedDataBits)}
              options={SERIAL_DATA_BITS}
              onUpdate={value => (selectedDataBits = value.value as DataBits)} />
            <UI.Select
              wrapperClass="w-28"
              label={{ name: "Parity" }}
              value={SERIAL_PARITY.find(a => a.value === selectedParity)}
              options={SERIAL_PARITY}
              onUpdate={value => (selectedParity = value.value as Parity)} />
            <UI.Select
              wrapperClass="w-28"
              label={{ name: "Data Bits", class: "px-0" }}
              value={SERIAL_STOP_BITS.find(a => a.value === selectedStopBits)}
              options={SERIAL_STOP_BITS}
              onUpdate={value => (selectedStopBits = value.value as StopBits)} />
            <!-- Настройки для протокола CANABLE -->
          {:else if selectedProtocol == "POECanable"}
            <UI.Select
              wrapperClass="w-28"
              label={{ name: "Bitrate" }}
              value={CAN_BITRATE.find(a => a.value === selectedCanBitrate)}
              options={CAN_BITRATE}
              onUpdate={value => (selectedCanBitrate = value.value as string)} />

            <!-- Настройки для протокола CANFD -->
          {:else if selectedProtocol == "POECanableFD"}
            <UI.Select
              wrapperClass="w-28"
              label={{ name: "Bitrate" }}
              value={CAN_FD_BITRATE.find(a => a.value === selectedCanFDBitrate)}
              options={CAN_FD_BITRATE}
              onUpdate={value => (selectedCanFDBitrate = value.value as string)} />
            <UI.Select
              wrapperClass="w-28"
              label={{ name: "Data Bitrate", class: "px-0" }}
              value={CAN_FD_DATA_BITRATE.find(a => a.value === selectedCanFDDataBitrate)}
              options={selectedCanFDBitrate == "S012F0C" ? CAN_FD_DATA_BITRATE.filter(value => value.value == "Y010B03") : CAN_FD_DATA_BITRATE}
              onUpdate={value => (selectedCanFDDataBitrate = value.value as string)} />
          {/if}
        </div>
      </div>
    {/snippet}
  </Block>

  <!-- COMMAND CREATOR -->
  <Block label="Command creator" isAccordion>
    {#snippet main()}
      <!-- Стандартные команды -->
      <div class="w-full">
        <div class="mx-1.5 flex items-center gap-1.5 py-2">
          <UI.Button
            wrapperClass="w-6.5"
            content={{ info: { text: "Hard restart", side: "top" }, icon: HardRestart }}
            componentClass="bg-red"
            onClick={async () =>
              await invoke("hard_restart", {
                path: connectedPort,
              })} />
          {#each defaultCommands as command}
            <UI.Button content={{ name: command.name }} wrapperClass="grow bg-gray" onClick={() => processDataSending(command)} />
          {/each}
          <UI.Button
            wrapperClass="w-8"
            content={{ icon: CommandList, info: { text: "Commands", side: "top" } }}
            onClick={() => (showCommandListModal = true)} />
        </div>
        <!-- Сохраненные команды -->
        <div class="flex flex-wrap items-center gap-1.5">
          {#if savedCommands}
            {#each savedCommands as command}
              {#if ((command.SimpleSerial && selectedProtocol == "SimpleSerial" && command.SimpleSerial.isShown) || (command.POESerial && selectedProtocol == "POESerial" && command.POESerial.isShown) || (command.POECanable && (selectedProtocol == "POECanable" || selectedProtocol == "POECanableFD") && command.POECanable.isShown)) && command.name}
                <UI.Button content={{ name: command.name }} wrapperClass="w-auto" componentClass="bg-purple" onClick={() => processDataSending(command)} />
              {/if}
            {/each}
          {/if}
        </div>
      </div>
    {/snippet}
    {#snippet details()}
      <div class={`flex items-end gap-2 `}>
        {#if selectedProtocol == "SimpleSerial"}
          <UI.Input label={{ name: "Regular String" }} bind:value={simpleSerialInputStr} />
          <UI.Select
            label={{ name: "End" }}
            wrapperClass="w-31"
            value={END_PACKAGE.find(e => e.value === selectedEndPackage)}
            options={END_PACKAGE}
            onUpdate={value => (selectedEndPackage = value.value as string)} />
        {:else if selectedProtocol == "POESerial"}
          <UI.Select
            label={{ name: "Header" }}
            wrapperClass="w-23"
            value={POE_SERIAL_HEADER.find(e => e.value == selectedPOESerialHeader)}
            options={POE_SERIAL_HEADER}
            onUpdate={value => (selectedPOESerialHeader = value.value as string)} />

          <UI.Select
            label={{ name: "Argument" }}
            type="input"
            wrapperClass="w-65"
            value={POE_SERIAL_ARGUMENT.find(e => e.value == selectedPOESerialArgument)}
            options={POE_SERIAL_ARGUMENT}
            onUpdate={value => (selectedPOESerialArgument = value.value as string)} />

          <UI.Input label={{ name: "Value" }} bind:value={POESerialValue} />
        {:else if selectedProtocol.startsWith("POECanable")}
          <UI.Select
            label={{ name: "Header", class: "p-0" }}
            wrapperClass="w-23"
            value={POE_CANABLE_HEADER.find(e => e.value == selectedCANableHeader)}
            options={POE_CANABLE_HEADER}
            onUpdate={value => {
              selectedCANableHeader = value.value as string
            }} />

          <UI.Select
            label={{ name: "Argument" }}
            type="input"
            wrapperClass="w-45"
            value={POE_CANABLE_ARGUMENT.find(e => e.value == selectedCANableArgument)}
            options={POE_CANABLE_ARGUMENT}
            onUpdate={value => (selectedCANableArgument = value.value as string)} />

          <UI.Input
            label={{ name: "TargetID", class: "!px-0" }}
            wrapperClass="w-20"
            bind:value={CANableTargetID}
            maxlength={3}
            help={{ regExp: /^(?:[1-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/ }} />
          <UI.Input
            label={{ name: "ReturnID", class: "!px-0" }}
            wrapperClass="w-20"
            bind:value={CANableReturnID}
            maxlength={3}
            help={{ regExp: /^(?:[1-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/ }} />
          <div class="flex flex-1 items-end">
            {#if CANableData}
              <UI.Switch
                label={{ name: "" }}
                options={[{ id: "ebc67545-7e4b-4b49-a074-e957c0d88431", value: 0, name: "0", class: "bg-red", disabled: false }]}
                bind:value={convertToBase64}
                type="checkbox"
                hiddenInfo="Convert data to Base64" />
            {/if}
            <UI.Input label={{ name: "Data", class: "!px-0" }} maxlength={200} bind:value={CANableData} />
          </div>
        {/if}

        <UI.Button wrapperClass="w-7 cursor-pointer mb-1" content={{ icon: SendCommand, info: { text: "Send", side: "top" } }} onClick={sendingCommand} />
      </div>
    {/snippet}
  </Block>

  <!-- DATA LOG -->
  <Block label="Data log" wrapperClass="flex-1">
    {#snippet main()}
      <div class={`absolute top-3.5 flex h-[calc(100%-1rem)] w-[calc(100%-1.5rem)] flex-col gap-2 text-left font-mono text-[11px] font-semibold`}>
        {#if showGraph}
          <div transition:slide={{ duration: 200 }}>
            <UI.Graph streamingData={{ data: dataForGraph, timestamp: timestampForGraph }} />
          </div>
        {/if}

        <UI.Table
          bind:this={tableRef}
          header={tableColumns}
          body={tableRows}
          bind:modalData
          dataBuffer={{
            stashData: selectedProtocol.startsWith("POECanable") ? false : true,
            rowsAmmount: 200,
            clearButton: selectedProtocol.startsWith("POECanable") ? false : true,
            clearClass: "w-8 top-0.5",
          }}
          autoscroll />
        {#if selectedProtocol.startsWith("POECanable")}
          <UI.Button wrapperClass="absolute w-8 bg-(--back-color) rounded-full p-1 right-2 top-0.75" content={{ icon: ButtonClear }} onClick={clearBuffers} />
        {/if}
      </div>

      <UI.Button
        wrapperClass="absolute -top-3 right-6 cursor-pointer bg-[#f5f7fa] h-6 w-8 items-center"
        content={{ icon: ShowGraph, info: { text: showGraph ? "Hide graph" : "Show graph", side: "top" } }}
        onClick={() => (showGraph = !showGraph)} />
    {/snippet}
  </Block>

  <UI.Modal isOpen={modalData.isOpen} title="Full data" wrapperClass="w-[80%] max-h-[80%]">
    {#snippet main()}
      <div class="text-left whitespace-pre">
        {@html modalData.formattedData}
      </div>
    {/snippet}
    {#snippet footer()}
      <UI.Button
        content={{ name: "Copy" }}
        wrapperClass="w-20 bg-pink"
        onClick={() => {
          navigator.clipboard.writeText(modalData.rawData)
          modalData.isOpen = false
        }} />
    {/snippet}
  </UI.Modal>

  <CommandListModal bind:showCommandListModal bind:savedCommands {selectedProtocol} />
</div>

<!-- Компактный вид -->
<button
  class={`relative ${isCollapsed ? "m-1.5 w-5 cursor-pointer p-3 opacity-100 select-none" : "m-0  h-0 w-0 p-0 opacity-0"} overflow-hidden rounded-xl border border-(--gray-color) inset-shadow-[0_0_3px_rgb(0_0_0_/0.75)] transition-all duration-200 ease-in-out`}
  onclick={() => (isCollapsed = false)}>
  <div class="absolute top-[50%] -left-3 rotate-270">
    {connectedPort ? connectedPort : " "}
  </div>
  <div class="dd"></div>
</button>

<style>
  /* Стили для светлой темы */
  .dddre {
    color: #333; /* Цвет текста для светлой темы */
    background: #f5f7fa;
  }

  /* Стили для темной темы */
  .uiwoew {
    color: #e2e3e7; /* Цвет текста для темной темы */
    background: #f5f7fa;
  }
</style>
