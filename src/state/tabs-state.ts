import { reactive } from 'vue'

import { v4 as uuid } from 'uuid'
import {
  AssemblyExecutionProfile,
  disassembleElf, ElfExecutionProfile,
  ExecutionProfile
} from '../utils/mips'

export interface EditorTab {
  uuid: string,
  title: string,
  lines: string[],
  breakpoints: number[],
  path: string | null,
  marked: boolean, // needs saving
  profile: ExecutionProfile | null
}

export function collectLines(lines: string[]): string {
  return lines.join('\n')
}

export interface TabsState {
  tabs: EditorTab[]
  selected: string | null
}

export const editor = reactive({
  tabs: [],
  selected: null,
  execution: null,
  debug: null
} as TabsState)

export function tab(): EditorTab | null {
  if (editor.selected) {
    return editor.tabs.find(tab => tab.uuid === editor.selected) ?? null
  }

  return null
}

export function closeTab(uuid: string) {
  const index = editor.tabs.findIndex(tab => tab.uuid === uuid)

  editor.tabs = editor.tabs.filter(tab => tab.uuid !== uuid)

  if (editor.selected === uuid) {
    if (editor.tabs.length < 0) {
      editor.selected = null

      return
    }

    const point = Math.min(Math.max(index - 1, 0), editor.tabs.length)

    editor.selected = editor.tabs[point].uuid
  }
}

function defaultAssemblyProfile(): AssemblyExecutionProfile  {
  return { kind: 'asm' }
}

export function createTab(
  named: string,
  content: string[],
  path: string | null = null,
  profile: ExecutionProfile | null = defaultAssemblyProfile()
) {
  const id = uuid()

  editor.tabs.push({
    uuid: id,
    title: named,
    lines: content,
    breakpoints: [],
    path,
    marked: false,
    profile
  })

  editor.selected = id
}

export async function loadElf(named: string, elf: ArrayBuffer) {
  const value = await disassembleElf(named, elf)

  const lines = value.error ? [value.error] : value.lines
  const profile = {
    kind: 'elf', elf, breakpoints: value.breakpoints
  } as ElfExecutionProfile

  createTab(named, lines, null, profile)
}

if (!editor.tabs.length) {
  createTab('Untitled', [''])
}
