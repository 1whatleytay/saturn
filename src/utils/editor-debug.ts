import { collectLines, tab } from '../state/editor-state'
import { consoleData, DebugTab, openConsole, pushConsole } from '../state/console-data'
import { ExecutionMode, ExecutionModeType, ExecutionResult, ExecutionState } from './mips'

import { format } from 'date-fns'

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
  consoleData.mode = result.mode.type
  
  switch (result.mode.type) {
    case ExecutionModeType.Finished: {
      const address = result.mode.value.toString(16).padStart(8, '0')

      pushConsole(`Execution finished at 0x${address}`)

      break
    }
    
    case ExecutionModeType.Invalid: {
      pushConsole(`Exception thrown: ${result.mode.value}`)

      consoleData.tab = DebugTab.Console

      break
    }

    default:
      break
  }
  
  consoleData.registers = {
    pc: result.pc,
    hi: result.hi,
    lo: result.lo,
    registers: result.registers
  }
}

function postBuildMessage(mode: ExecutionMode) {
  switch (mode.type) {
    case ExecutionModeType.BuildFailed:
      const error = mode.value

      const marker = error.marker ? ` (line ${error.marker.line})` : ''
      const trailing = error.body ? `\n${error.body}` : ''

      openConsole(`Build failed: ${error.message}${marker}${trailing}`)

      break

    default:
      openConsole(`Build succeeded at ${format(Date.now(), 'MMMM d, p')}`)

      break
  }

  consoleData.tab = DebugTab.Console
}

export async function resume() {
  const usedProfile = tab()?.profile
  const usedBreakpoints = tab()?.breakpoints ?? []

  if (!usedProfile) {
    return
  }

  let needsBuild = false
  if (!consoleData.execution) {
    const text = collectLines(tab()?.lines ?? [])

    needsBuild = true
    consoleData.execution = new ExecutionState(text, usedProfile)
  }

  consoleData.showConsole = true
  consoleData.mode = ExecutionModeType.Running

  const result = await consoleData.execution.resume(usedBreakpoints)

  consoleData.showConsole = true

  if (needsBuild) {
    postBuildMessage(result.mode)
  }

  postDebugInformation(result)
}

export async function pause() {
  if (consoleData.execution) {
    consoleData.showConsole = true

    postDebugInformation(await consoleData.execution.pause())
  }
}

export async function step() {
  if (consoleData.execution) {
    consoleData.showConsole = true

    postDebugInformation(await consoleData.execution.step())
  }
}

export async function stop() {
  if (consoleData.execution) {
    await consoleData.execution.stop()

    consoleData.execution = null
  }
}
