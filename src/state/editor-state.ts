import { reactive } from 'vue'

import { v4 as uuid } from 'uuid'
import { disassembleElf } from '../utils/disassemble'

export interface EditorTab {
  uuid: string,
  title: string,
  lines: string[]
}

export interface EditorState {
  tabs: EditorTab[],
  selected: string | null
}

export const state = reactive({
  tabs: [],
  selected: null
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

export function createTab(named: string, content: string[]) {
  const id = uuid()

  state.tabs.push({
    uuid: id,
    title: named,
    lines: content
  })

  state.selected = id
}

export async function loadElf(named: string, elf: ArrayBuffer) {
  createTab(named, await disassembleElf(elf))
}

if (!state.tabs.length) {
  createTab('Untitled', [''])
}
