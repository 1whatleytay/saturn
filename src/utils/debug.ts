import { collectLines } from './tabs'
import {
  consoleData,
  ConsoleType,
  DebugTab,
  openConsole,
  pushConsole
} from '../state/console-data'
import { backend } from '../state/backend'
import {
  AssemblerResult,
  ExecutionModeType,
  ExecutionResult
} from './mips/mips'
import { tab, settings, buildLines } from '../state/state'

import { format } from 'date-fns'
import { PromptType, saveCurrentTab } from './events'

export async function setBreakpoint(line: number, remove: boolean) {
  const currentTab = tab()

  if (!currentTab) {
    return
  }

  if (remove) {
    currentTab.breakpoints = currentTab.breakpoints.filter(
      (point) => point !== line
    )
  } else if (!currentTab.breakpoints.includes(line)) {
    currentTab.breakpoints.push(line)
  }

  if (consoleData.execution) {
    await consoleData.execution.setBreakpoints(currentTab.breakpoints)
  }
}

async function postDebugInformationWithPcHint(result: ExecutionResult) {
  postDebugInformation(result)

  const execution = consoleData.execution

  if (execution && !(execution.breakpoints?.pcToGroup.has(result.registers.pc) ?? true)) {
    consoleData.hintPc = await execution.lastPc()
  }
}

function postDebugInformation(result: ExecutionResult) {
  if (consoleData.mode === ExecutionModeType.Stopped) {
    return
  }

  consoleData.hintPc = null
  consoleData.mode = result.mode.type
  consoleData.registers = result.registers

  switch (result.mode.type) {
    case ExecutionModeType.Finished: {
      const address = result.mode.pc.toString(16).padStart(8, '0')

      if (result.mode.code !== null) {
        pushConsole(
          `Execution finished with code ${result.mode.code} at pc 0x${address}`,
          ConsoleType.Success
        )
      } else {
        pushConsole(
          `Execution finished at pc 0x${address}`,
          ConsoleType.Success
        )
      }

      closeExecution()

      break
    }

    case ExecutionModeType.Invalid: {
      pushConsole(`Exception thrown: ${result.mode.message}`, ConsoleType.Error)

      consoleData.tab = DebugTab.Console

      break
    }

    default:
      break
  }
}

function clearDebug() {
  consoleData.registers = null
}

function closeExecution() {
  consoleData.execution = null
}

export function postBuildMessage(result: AssemblerResult): boolean {
  switch (result.status) {
    case 'Error':
      consoleData.mode = null
      consoleData.tab = DebugTab.Console
      consoleData.showConsole = true

      const marker = result.marker ? ` (line ${result.marker.line + 1})` : ''
      const trailing = result.body
        ? `\n${result.body
            .split('\n')
            .map((x) => `> ${x}`)
            .join('\n')}`
        : ''

      openConsole()
      pushConsole(
        `Build failed: ${result.message}${marker}${trailing}`,
        ConsoleType.Error
      )

      return false

    case 'Success':
      if (!consoleData.showConsole) {
        consoleData.tab = DebugTab.Console
        consoleData.showConsole = true
      }

      openConsole()
      pushConsole(
        `Build succeeded at ${format(Date.now(), 'MMMM d, pp')}`,
        ConsoleType.Success
      )

      return true
  }
}

export async function build() {
  await saveCurrentTab(PromptType.NeverPrompt)

  const current = tab()

  const {
    binary,
    result
  } = await backend.assembleWithBinary(collectLines(current?.lines ?? []), current?.path ?? null)

  if (binary !== null) {
    buildLines.value = await backend.disassemblyDetails(binary.buffer)
  }

  consoleData.showConsole = true
  consoleData.tab = DebugTab.Console
  postBuildMessage(result)
}

export async function resume() {
  clearDebug()

  const current = tab()

  if (!current || !current.profile) {
    return
  }

  const usedBreakpoints = current.breakpoints ?? []

  if (!consoleData.execution) {
    const text = collectLines(current.lines ?? [])
    const path = current.path

    await saveCurrentTab(PromptType.NeverPrompt)

    consoleData.execution = await backend.createExecution(text, path, settings.execution.timeTravel, current.profile)
  }

  consoleData.showConsole = true
  consoleData.mode = ExecutionModeType.Running

  const result = await consoleData.execution.resume(
    null,
    usedBreakpoints,
    (result) => postBuildMessage(result)
  )

  if (result) {
    await postDebugInformationWithPcHint(result)
  } else {
    closeExecution()
  }
}

export async function pause() {
  if (!consoleData.execution) {
    return
  }

  consoleData.showConsole = true

  await consoleData.execution.pause()
}

export async function stepCount(skip: number) {
  if (!consoleData.execution) {
    return
  }

  clearDebug()
  consoleData.mode = ExecutionModeType.Running

  const result = await consoleData.execution.resume(skip, null, () => {})

  consoleData.showConsole = true

  if (result) {
    await postDebugInformationWithPcHint(result)
  }
}

export async function step() {
  if (!consoleData.execution) {
    return
  }

  const breakpoints = consoleData.execution.breakpoints

  let skip = 1
  if (breakpoints && consoleData.registers) {
    const pc = consoleData.registers.pc
    const group = breakpoints.pcToGroup.get(pc)

    if (group) {
      const index = group.pcs.findIndex((x) => x === pc)

      if (index >= 0) {
        skip = group.pcs.length - index
      }
    }
  }

  await stepCount(skip)
}

export async function rewind() {
  if (!consoleData.execution || !consoleData.execution.timeTravel) {
    return
  }

  const breakpoints = consoleData.execution.breakpoints

  const pc = await consoleData.execution.lastPc()

  let skip = 1
  if (breakpoints && pc !== null) {
    const group = breakpoints.pcToGroup.get(pc)

    if (group) {
      const index = group.pcs.findIndex((x) => x === pc)

      if (index >= 0) {
        skip = index + 1
      }
    }
  }

  clearDebug()
  consoleData.mode = ExecutionModeType.Running

  const result = await consoleData.execution.rewind(skip)

  consoleData.showConsole = true

  if (result) {
    await postDebugInformationWithPcHint(result)
  }
}

export async function stop() {
  if (!consoleData.execution) {
    return
  }

  consoleData.mode = ExecutionModeType.Stopped

  await consoleData.execution.stop()

  closeExecution()
}

export async function setRegister(id: number, value: number) {
  if (consoleData.execution && consoleData.registers) {
    switch (id) {
      case 32:
        consoleData.registers.hi = value
        break

      case 33:
        consoleData.registers.lo = value
        break

      case 34:
        consoleData.registers.pc = value
        break

      default:
        consoleData.registers.line[id] = value
        break
    }

    await consoleData.execution.setRegister(id, value)
  }
}
