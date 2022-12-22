import { reactive } from 'vue'

import { v4 as uuid } from 'uuid'
import { disassembleElf, ExecutionProfile, ExecutionState } from '../utils/mips'

export interface EditorTab {
  uuid: string,
  title: string,
  lines: string[],
  breakpoints: number[],
  profile: ExecutionProfile | null
}

export interface EditorState {
  tabs: EditorTab[],
  selected: string | null,
  execution: ExecutionState | null,
  debug: string | null
}

export const state = reactive({
  tabs: [],
  selected: null,
  execution: null,
  debug: null
} as EditorState)

export function tab(): EditorTab | null {
  if (state.selected) {
    return state.tabs.find(tab => tab.uuid === state.selected) ?? null
  }

  return null
}

export function remove(uuid: string) {
  state.tabs = state.tabs.filter(tab => tab.uuid !== uuid)

  if (state.selected === uuid) {
    state.selected = null
  }
}

export function createTab(named: string, content: string[], profile: ExecutionProfile | null = null) {
  const id = uuid()

  state.tabs.push({
    uuid: id,
    title: named,
    lines: content,
    breakpoints: [],
    profile
  })

  state.selected = id
}

export async function loadElf(named: string, elf: ArrayBuffer) {
  const value = await disassembleElf(named, elf)

  const lines = value.error ? [value.error] : value.lines

  createTab(named, lines, { elf, breakpoints: value.breakpoints })
}

if (!state.tabs.length) {
  createTab('Untitled', [''])
}
