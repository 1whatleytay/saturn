import { reactive } from 'vue'
import { ExecutionModeType, ExecutionState, Registers } from '../utils/mips'

import { v4 as uuid } from 'uuid'

export enum DebugTab {
  Registers,
  Memory,
  Console,
  Bitmap
}

interface ConsoleBlock {
  id: string
  text: string
  highlight: string // tailwind color :)
}

interface ConsoleData {
  showConsole: boolean
  execution: ExecutionState | null
  mode: ExecutionModeType | null,
  registers: Registers | null
  tab: DebugTab
  console: ConsoleBlock[]
}

export const consoleData = reactive({
  showConsole: false,
  execution: null,
  mode: null,
  registers: null,
  tab: DebugTab.Registers,
  console: []
} as ConsoleData)

export function openConsole(text: string) {
  consoleData.console = []

  pushConsole(text)
}

export function pushConsole(text: string) {
  text.split('\n').forEach(line => {
    consoleData.console.push({
      id: uuid(),
      text: line,
      highlight: 'text-teal-100'
    })
  })
}
