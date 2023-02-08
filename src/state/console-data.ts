import { reactive } from 'vue'
import { ExecutionModeType, ExecutionState, Registers } from '../utils/mips'

export enum DebugTab {
  Registers,
  Memory,
  Console,
  Bitmap
}

interface ConsoleLineMeta {
  highlight: string
}

interface ConsoleData {
  showConsole: boolean
  execution: ExecutionState | null
  mode: ExecutionModeType | null,
  registers: Registers | null
  tab: DebugTab
  console: string[]
  consoleMeta: Map<number, ConsoleLineMeta> // index in console[] -> Meta
}

export const consoleData = reactive({
  showConsole: false,
  execution: null,
  mode: null,
  registers: null,
  tab: DebugTab.Registers,
  console: new Array(500)
    .fill(0)
    .map((_, i) => `Hello World, this is a test ${i}`),
  consoleMeta: new Map()
} as ConsoleData)

export function openConsole(text: string) {
  consoleData.console = []

  pushConsole(text)
}

export function pushConsole(text: string) {
  text.split('\n').forEach(line => {
    const index = consoleData.console.length

    consoleData.console.push(line)
    consoleData.consoleMeta.set(index, { highlight: 'text-teal-100' })
  })
}
