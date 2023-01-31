import { collectLines, tab } from '../state/tabs-state'
import { consoleData, DebugTab, openConsole, pushConsole } from '../state/console-data'
import {
  AssemblerResult,
  assembleText,
  ExecutionModeType,
  ExecutionResult,
  ExecutionState
} from './mips'

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
        pushConsole(`Execution finished with code ${result.mode.code} at pc 0x${address}`)
      } else {
        pushConsole(`Execution finished at pc 0x${address}`)
      }

      closeExecution()

      break
    }
    
    case ExecutionModeType.Invalid: {
      pushConsole(`Exception thrown: ${result.mode.message}`)

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
  clearDebug()

  consoleData.execution = null
}

export function postBuildMessage(result: AssemblerResult): boolean {
  consoleData.tab = DebugTab.Console

  switch (result.status) {
    case 'Error':
      const marker = result.marker ? ` (line ${result.marker.line + 1})` : ''
      const trailing = result.body
        ? `\n${result.body.split('\n').map(x => `> ${x}`).join('\n')}`
        : ''

      openConsole(`Build failed: ${result.message}${marker}${trailing}`)

      return false

    case 'Success':
      openConsole(`Build succeeded at ${format(Date.now(), 'MMMM d, pp')}`)

      return true
  }
}

export async function build() {
  await saveCurrentTab(PromptType.NeverPrompt)

  const result = await assembleText(collectLines(tab()?.lines ?? []))

  consoleData.showConsole = true
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

  const result = await consoleData.execution.resume(usedBreakpoints, result => {
    postBuildMessage(result)
  })

  consoleData.showConsole = true

  if (result) {
    postDebugInformation(result)
  } else {
    closeExecution()
  }
}

export async function pause() {
  if (consoleData.execution) {
    clearDebug()

    consoleData.showConsole = true

    postDebugInformation(await consoleData.execution.pause())
  }
}

export async function step() {
  if (consoleData.execution) {
    clearDebug()

    consoleData.showConsole = true

    postDebugInformation(await consoleData.execution.step())
  }
}

export async function stop() {
  if (consoleData.execution) {
    consoleData.mode = ExecutionModeType.Stopped

    await consoleData.execution.stop()

    closeExecution()
  }
}
