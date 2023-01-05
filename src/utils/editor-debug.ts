import { collectLines, state, tab } from './editor-state'
import { defaultResult, ExecutionMode, ExecutionState } from '../utils/mips'

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

  if (state.execution) {
    await state.execution.setBreakpoints(currentTab.breakpoints)
  }
}

export async function resume() {
  const usedProfile = tab()?.profile
  const usedBreakpoints = tab()?.breakpoints ?? []

  if (!usedProfile) {
    return
  }

  if (!state.execution) {
    const text = collectLines(tab()?.lines ?? [])

    state.execution = new ExecutionState(text, usedProfile)
  }

  // TODO: On set breakpoint while execution is non-null:
  //  - await state.execution.pause()
  //  - await state.execution.resume(newBreakpoints)

  state.execution.resume(usedBreakpoints)
    .then(result => {
      state.debug = result
    })

  state.debug = defaultResult(ExecutionMode.Running)
}

export async function pause() {
  if (state.execution) {
    state.debug = await state.execution.pause()
  }
}

export async function step() {
  if (state.execution) {
    state.debug = await state.execution.step()
  }
}

export async function stop() {
  if (state.execution) {
    await state.execution.stop()

    state.execution = null
  }
}