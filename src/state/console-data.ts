import { reactive } from 'vue'
import { ExecutionModeType, ExecutionState } from '../utils/mips'

export enum DebugTab {
  Registers,
  Memory,
  Console,
  Bitmap
}

export interface Registers {
  pc: number,
  registers: number[],
  lo: number,
  hi: number,
}

interface ConsoleData {
  showConsole: boolean
  execution: ExecutionState | null
  mode: ExecutionModeType | null,
  registers: Registers | null
  tab: DebugTab
  console: string
}

export const consoleData = reactive({
  showConsole: false,
  execution: null,
  mode: null,
  registers: null,
  tab: DebugTab.Registers,
  console: ''
} as ConsoleData)

export function openConsole(text: string, terminator: string = '\n') {
  consoleData.console = ''

  pushConsole(text, terminator)
}

export function pushConsole(text: string, terminator: string = '\n') {
  consoleData.console += `${text}${terminator}`
}
