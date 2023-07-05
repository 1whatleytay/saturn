import { reactive } from 'vue'
import { ExecutionModeType, ExecutionState, Registers } from '../utils/mips'

export enum DebugTab {
  Registers,
  Memory,
  Console,
  Bitmap,
  Tests,
}

export enum ConsoleType {
  Stdout,
  Stderr,
  Success,
  Error,
  Info,
  Secondary,
  Editing,
  Submitted,
}

function canConcat(value: ConsoleType): boolean {
  switch (value) {
    case ConsoleType.Stdout:
    case ConsoleType.Stderr:
      return true

    default:
      return false
  }
}

interface ConsoleLineMeta {
  type: ConsoleType
}

interface ConsoleData {
  showConsole: boolean
  execution: ExecutionState | null
  mode: ExecutionModeType | null
  registers: Registers | null
  tab: DebugTab
  console: string[]
  consoleMeta: Map<number, ConsoleLineMeta> // index in console[] -> Meta
}

const editHighlight = { type: ConsoleType.Editing }
const submitHighlight = { type: ConsoleType.Submitted }
const secondaryHighlight = { type: ConsoleType.Secondary }

export const consoleData = reactive({
  showConsole: false,
  execution: null,
  mode: null,
  registers: null,
  tab: DebugTab.Console,
  console: ['Nothing yet.', '', ''],
  consoleMeta: new Map([
    [0, secondaryHighlight],
    [1, secondaryHighlight],
    [2, editHighlight],
  ]),
} as ConsoleData)

export function openConsole() {
  consoleData.console = ['']
  consoleData.consoleMeta = new Map([[0, editHighlight]])
}

export function pushConsole(text: string, type: ConsoleType): number {
  const count = consoleData.console.length
  const meta = consoleData.consoleMeta.get(count)
  const editLine = count ? consoleData.console[count - 1] : null

  const concat =
    canConcat(type) && consoleData.consoleMeta.get(count - 2)?.type === type
  const startIndex = concat ? 1 : 0

  let overwrote = false
  text.split('\n').forEach((line, index) => {
    let point: number

    if (concat && index === 0) {
      consoleData.console[count - 2] += line
      point = count - 2 // ?
    } else if (index === startIndex) {
      overwrote = true
      consoleData.console[count - 1] = line
      point = count - 1
    } else {
      point = consoleData.console.length
      consoleData.console.push(line)
    }

    consoleData.consoleMeta.set(point, { type })
  })

  if (overwrote) {
    const lastIndex = consoleData.console.length

    consoleData.console.push(editLine ?? '')
    consoleData.consoleMeta.set(lastIndex, meta ?? editHighlight)
  }

  return count - 1
}

// returns if the submission went through
export function submitConsole(force: boolean = false): boolean {
  const count = consoleData.console.length

  if (count <= 0) {
    return false
  }

  const last = consoleData.console[count - 1]

  if (!force && last.length <= 0) {
    return false
  }

  consoleData.consoleMeta.set(count - 1, submitHighlight)

  consoleData.console.push('')
  consoleData.consoleMeta.set(count, editHighlight)

  if (consoleData.execution) {
    consoleData.execution.postInput(`${last}\n`).then(() => {})
  }

  return true
}
