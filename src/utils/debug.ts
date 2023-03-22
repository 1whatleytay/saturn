import { collectLines } from './tabs'
import { consoleData, ConsoleType, DebugTab, openConsole, pushConsole } from '../state/console-data'
import { AssemblerResult, assembleText, ExecutionModeType, ExecutionResult, ExecutionState } from './mips'
import { tab } from '../state/state'

import { format } from 'date-fns'
import { PromptType, saveCurrentTab } from './events'

export async function setBreakpoint(line: number, remove: boolean) {
  const currentTab = tab()

  if (!currentTab) {
    return
  }

  if (remove) {
    currentTab.breakpoints = currentTab.breakpoints
      .filter(point => point !== line)
  } else if (!currentTab.breakpoints.includes(line)) {
    currentTab.breakpoints.push(line)
  }

  if (consoleData.execution) {
    await consoleData.execution.setBreakpoints(currentTab.breakpoints)
  }
}

function postDebugInformation(result: ExecutionResult) {
  if (consoleData.mode === ExecutionModeType.Stopped) {
    return
  }

  consoleData.mode = result.mode.type
  consoleData.registers = result.registers
  
  switch (result.mode.type) {
    case ExecutionModeType.Finished: {
      const address = result.mode.pc.toString(16).padStart(8, '0')

      if (result.mode.code !== null) {
        pushConsole(`Execution finished with code ${result.mode.code} at pc 0x${address}`, ConsoleType.Success)
      } else {
        pushConsole(`Execution finished at pc 0x${address}`, ConsoleType.Success)
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
        ? `\n${result.body.split('\n').map(x => `> ${x}`).join('\n')}`
        : ''

      openConsole()
      pushConsole(`Build failed: ${result.message}${marker}${trailing}`, ConsoleType.Error)

      return false

    case 'Success':
      if (!consoleData.showConsole) {
        consoleData.tab = DebugTab.Console
        consoleData.showConsole = true
      }

      openConsole()
      pushConsole(`Build succeeded at ${format(Date.now(), 'MMMM d, pp')}`, ConsoleType.Success)

      return true
  }
}

export async function build() {
  await saveCurrentTab(PromptType.NeverPrompt)

  const result = await assembleText(collectLines(tab()?.lines ?? []))

  consoleData.showConsole = true
  consoleData.tab = DebugTab.Console
  postBuildMessage(result)
}

export async function resume() {
  clearDebug()

  const usedProfile = tab()?.profile
  const usedBreakpoints = tab()?.breakpoints ?? []

  if (!usedProfile) {
    return
  }

  if (!consoleData.execution) {
    const text = collectLines(tab()?.lines ?? [])

    await saveCurrentTab(PromptType.NeverPrompt)

    consoleData.execution = new ExecutionState(text, usedProfile)
  }

  consoleData.showConsole = true
  consoleData.mode = ExecutionModeType.Running

  const result = await consoleData.execution.resume(
    null, usedBreakpoints, result => postBuildMessage(result)
  )

  if (result) {
    postDebugInformation(result)
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
      const index = group.pcs.findIndex(x => x === pc)

      if (index >= 0) {
        skip = group.pcs.length - index
      }
    }
  }

  clearDebug()
  consoleData.mode = ExecutionModeType.Running

  const result = await consoleData.execution.resume(skip, null)

  consoleData.showConsole = true

  if (result) {
    postDebugInformation(result)
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
