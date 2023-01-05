import { reactive } from 'vue'
import { ExecutionResult, ExecutionState } from '../utils/mips'

interface ConsoleData {
  showConsole: boolean,
  execution: ExecutionState | null
  debug: ExecutionResult | null
}

export const consoleData = reactive({
  showConsole: false,
  execution: null,
  debug: null
} as ConsoleData)
