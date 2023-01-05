import { collectLines, tab } from '../state/editor-state'
import { consoleData } from '../state/console-data'
import { defaultResult, ExecutionMode, ExecutionState } from './mips'

export async function setBreakpoint(line: number, remove: boolean) {
  const currentTab = tab()

  if (!currentTab) {
    return
  }

  if (remove) {
    currentTab.breakpoints = currentTab.breakpoints
      .filter(point => point !== line)
  } else if (currentTab.breakpoints.includes(line)) {
    currentTab.breakpoints.push(line)
  }

  if (consoleData.execution) {
    await consoleData.execution.setBreakpoints(currentTab.breakpoints)
  }
}

export async function resume() {
  const usedProfile = tab()?.profile
  const usedBreakpoints = tab()?.breakpoints ?? []

  if (!usedProfile) {
    return
  }

  if (!consoleData.execution) {
    const text = collectLines(tab()?.lines ?? [])

    consoleData.execution = new ExecutionState(text, usedProfile)
  }

  // TODO: On set breakpoint while execution is non-null:
  //  - await state.execution.pause()
  //  - await state.execution.resume(newBreakpoints)

  consoleData.execution.resume(usedBreakpoints)
    .then(result => {
      consoleData.showConsole = true
      consoleData.debug = result
    })

  consoleData.showConsole = true
  consoleData.debug = defaultResult(ExecutionMode.Running)
}

export async function pause() {
  if (consoleData.execution) {
    consoleData.showConsole = true
    consoleData.debug = await consoleData.execution.pause()
  }
}

export async function step() {
  if (consoleData.execution) {
    consoleData.showConsole = true
    consoleData.debug = await consoleData.execution.step()
  }
}

export async function stop() {
  if (consoleData.execution) {
    await consoleData.execution.stop()

    consoleData.execution = null
  }
}