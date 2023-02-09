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

const editHighlight = { highlight: 'text-green-400 bg-green-900 bg-opacity-10' }
const submitHighlight = { highlight: 'text-green-500 font-bold' }

export const consoleData = reactive({
  showConsole: false,
  execution: null,
  mode: null,
  registers: null,
  tab: DebugTab.Registers,
  console: [''],
  consoleMeta: new Map([[0, editHighlight]])
} as ConsoleData)

export function openConsole(text: string) {
  consoleData.console = ['']
  consoleData.consoleMeta = new Map([[0, editHighlight]])

  pushConsole(text)
}

export function pushConsole(text: string) {
  const count = consoleData.console.length
  const meta = consoleData.consoleMeta.get(count)
  const editLine = count ? consoleData.console[count - 1] : null

  text.split('\n').forEach((line, index) => {
    let point: number

    if (index === 0) {
      consoleData.console[count - 1] = line
      point = count - 1
    } else {
      consoleData.console.push()
      point = consoleData.console.length
    }

    consoleData.consoleMeta.set(point, { highlight: 'text-teal-100' })
  })

  const lastIndex = consoleData.console.length

  consoleData.console.push(editLine ?? '')
  consoleData.consoleMeta.set(lastIndex, meta ?? editHighlight)
}

// returns if the submission went through
export function submitConsole(): boolean {
  const count = consoleData.console.length

  if (count <= 0) {
    return false
  }

  const last = consoleData.console[count - 1]

  consoleData.consoleMeta.set(count - 1, submitHighlight)

  consoleData.console.push('')
  consoleData.consoleMeta.set(count, editHighlight)

  if (consoleData.execution) {
    consoleData.execution.postInput(last).then(() => { })
  }

  return true
}
