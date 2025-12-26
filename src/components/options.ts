import { ARGUMENT_MAP, HEADER_MAP } from "../stores/Dictionary"
import type { ISelectOption } from "../stores/Interfaces"

export const PORT_RROTOCOL: ISelectOption[] = [
  {
    id: crypto.randomUUID(),
    name: "Simple Serial",
    value: "SimpleSerial",
  },
  { id: crypto.randomUUID(), name: "POE Serial", value: "POESerial" },
  { id: crypto.randomUUID(), name: "POE CANable", value: "POECanable" },
  { id: crypto.randomUUID(), name: "POE CAN-FD", value: "POECanableFD" },
]

export const SERIAL_BAUD_RATE: ISelectOption[] = [
  { id: crypto.randomUUID(), name: "600", value: 600 },
  { id: crypto.randomUUID(), name: "1200", value: 1200 },
  { id: crypto.randomUUID(), name: "2400", value: 2400 },
  { id: crypto.randomUUID(), name: "4800", value: 4800 },
  { id: crypto.randomUUID(), name: "9600", value: 9600 },
  { id: crypto.randomUUID(), name: "14400", value: 14400 },
  { id: crypto.randomUUID(), name: "19200", value: 19200 },
  { id: crypto.randomUUID(), name: "28800", value: 28800 },
  { id: crypto.randomUUID(), name: "38400", value: 38400 },
  { id: crypto.randomUUID(), name: "56000", value: 56000 },
  { id: crypto.randomUUID(), name: "57600", value: 57600 },
  { id: crypto.randomUUID(), name: "115200", value: 115200 },
  { id: crypto.randomUUID(), name: "128000", value: 128000 },
  { id: crypto.randomUUID(), name: "256000", value: 256000 },
]

export const SERIAL_DATA_BITS: ISelectOption[] = [
  { id: crypto.randomUUID(), name: "5", value: "5" },
  { id: crypto.randomUUID(), name: "6", value: "6" },
  { id: crypto.randomUUID(), name: "7", value: "7" },
  { id: crypto.randomUUID(), name: "8", value: "8" },
]

export const SERIAL_PARITY: ISelectOption[] = [
  { id: crypto.randomUUID(), name: "none", value: "0" },
  { id: crypto.randomUUID(), name: "odd", value: "1" },
  { id: crypto.randomUUID(), name: "even", value: "2" },
]

export const SERIAL_STOP_BITS: ISelectOption[] = [
  { id: crypto.randomUUID(), name: "1", value: "1" },
  { id: crypto.randomUUID(), name: "2", value: "2" },
]
export const CAN_BITRATE: ISelectOption[] = [
  { id: crypto.randomUUID(), name: "10k", value: "S0" },
  { id: crypto.randomUUID(), name: "20k", value: "S1" },
  { id: crypto.randomUUID(), name: "50k", value: "S2" },
  { id: crypto.randomUUID(), name: "100k", value: "S3" },
  { id: crypto.randomUUID(), name: "125k", value: "S4" },
  { id: crypto.randomUUID(), name: "250k", value: "S5" },
  { id: crypto.randomUUID(), name: "500k", value: "S6" },
  { id: crypto.randomUUID(), name: "750k", value: "S7" },
  { id: crypto.randomUUID(), name: "1M", value: "S8" },
]

export const CAN_FD_BITRATE: ISelectOption[] = [
  // { id: crypto.randomUUID(), name: '10k', value: 'S0' },
  // { id: crypto.randomUUID(), name: '20k', value: 'S1' },
  // { id: crypto.randomUUID(), name: '50k', value: 'S2' },
  // { id: crypto.randomUUID(), name: '100k', value: 'S3' },
  // { id: crypto.randomUUID(), name: '125k', value: 'S4' },
  { id: crypto.randomUUID(), name: "250k", value: "S042F0C" },
  // { id: crypto.randomUUID(), name: '500k', value: 'S6' },
  { id: crypto.randomUUID(), name: "500k", value: "S022F0C" },
  // { id: crypto.randomUUID(), name: '750k', value: 'S7' },
  { id: crypto.randomUUID(), name: "1M", value: "S012F0C" },
  // { id: crypto.randomUUID(), name: '83.3k', value: 'S9' },
  // { id: crypto.randomUUID(), name: '75k', value: 'SA' },
  // { id: crypto.randomUUID(), name: '62.5k', value: 'SB' },
  // { id: crypto.randomUUID(), name: '33.3k', value: 'SC' },
  // { id: crypto.randomUUID(), name: '5k', value: 'SD' },
]

export const CAN_FD_DATA_BITRATE: ISelectOption[] = [
  { id: crypto.randomUUID(), name: "1M", value: "Y021706" },
  // { id: crypto.randomUUID(), name: '2M', value: 'Y2' },
  { id: crypto.randomUUID(), name: "2M", value: "Y011706" },
  // { id: crypto.randomUUID(), name: '3M', value: 'Y3' },
  { id: crypto.randomUUID(), name: "4M", value: "Y010B03" },
  // { id: crypto.randomUUID(), name: '5M', value: 'Y5' },
]

export const END_PACKAGE: ISelectOption[] = [
  { id: crypto.randomUUID(), name: "LF (n)", value: "\n" },
  { id: crypto.randomUUID(), name: "CR (r)", value: "\r" },
  { id: crypto.randomUUID(), name: "CRLF (rn)", value: "\r\n" },
  { id: crypto.randomUUID(), name: "None", value: "" },
]

export const POE_SERIAL_HEADER: ISelectOption[] = Object.entries(HEADER_MAP).map(([name]) => ({
  id: crypto.randomUUID(),
  name,
  value: name,
}))

export const POE_SERIAL_ARGUMENT: ISelectOption[] = [
  ...Object.entries(ARGUMENT_MAP).map(([name]) => ({
    id: crypto.randomUUID(),
    name,
    value: name,
  })),
]

export const POE_CANABLE_HEADER: ISelectOption[] = [
  ...Object.entries(HEADER_MAP).map(([name, value]) => ({
    id: crypto.randomUUID(),
    name,
    value,
  })),
]

export const POE_CANABLE_ARGUMENT: ISelectOption[] = [
  ...Object.entries(ARGUMENT_MAP).map(([name, value]) => ({
    id: crypto.randomUUID(),
    name,
    value,
  })),
]
