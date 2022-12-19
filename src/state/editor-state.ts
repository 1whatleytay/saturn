import { watch, reactive } from 'vue'

import { tauri } from '@tauri-apps/api'

import { v4 as uuid } from 'uuid'

export interface EditorTab {
  uuid: string,
  title: string,
  lines: string[]
}

export interface EditorState {
  tabs: EditorTab[],
  selected: string | null
}

const defaultTabs = [
  { uuid: uuid(), title: 'One', lines: ['a', 'b', 'c', 'd'] },
  { uuid: uuid(), title: 'Two', lines: ['e', 'fg', 'h', 'i'] },
  { uuid: uuid(), title: 'Three', lines: ['j', 'k', 'l', 'm'] }
] as EditorTab[]

export const state = reactive({
  tabs: defaultTabs,
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

export async function disassembleElf(elf: ArrayBuffer): Promise<string[]> {
  const bytes = Array.from(new Uint8Array(elf))

  return tauri.invoke('disassemble', { bytes })
}

export async function loadElf(named: string, elf: ArrayBuffer) {
  const id = uuid()

  state.tabs.push({
    uuid: id,
    title: named,
    lines: await disassembleElf(elf)
  })

  state.selected = id
}

// export function listen(state: EditorState, key: string = 'editor', storage: Storage = localStorage) {
//   watch(() => state, (value) => {
//     storage.setItem(key, JSON.stringify(value))
//   })
// }
//
// listen(state)
