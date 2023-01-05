import { reactive } from 'vue'
import { ExecutionResult, ExecutionState } from '../utils/mips'

export enum DebugTab {
  Registers,
  Memory,
  Console
}

interface ConsoleData {
  showConsole: boolean,
  execution: ExecutionState | null
  debug: ExecutionResult | null,
  tab: DebugTab,
  console: string
}

export const consoleData = reactive({
  showConsole: false,
  execution: null,
  debug: null,
  tab: DebugTab.Registers,
  console: ''
} as ConsoleData)

export function openConsole(text: string) {
  consoleData.console = text
}

export function pushConsole(text: string, terminator: string = '\n') {
  consoleData.console += `${text}${terminator}`
}
