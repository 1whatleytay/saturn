import { reactive } from 'vue'

import { v4 as uuid } from 'uuid'
import {
  AssemblyExecutionProfile,
  disassembleElf,
  ExecutionProfile
} from '../utils/mips'

export interface EditorTab {
  uuid: string,
  title: string,
  lines: string[],
  breakpoints: number[],
  profile: ExecutionProfile | null
}

export function collectLines(lines: string[]): string {
  return lines.join('\n')
}

export interface EditorState {
  tabs: EditorTab[]
  selected: string | null
}

export const editor = reactive({
  tabs: [],
  selected: null,
  execution: null,
  debug: null
} as EditorState)

export function tab(): EditorTab | null {
  if (editor.selected) {
    return editor.tabs.find(tab => tab.uuid === editor.selected) ?? null
  }

  return null
}

export function remove(uuid: string) {
  editor.tabs = editor.tabs.filter(tab => tab.uuid !== uuid)

  if (editor.selected === uuid) {
    editor.selected = null
  }
}

function defaultAssemblyProfile(): AssemblyExecutionProfile  {
  return { kind: 'asm' }
}

export function createTab(
  named: string,
  content: string[],
  profile: ExecutionProfile | null = defaultAssemblyProfile()
) {
  const id = uuid()

  editor.tabs.push({
    uuid: id,
    title: named,
    lines: content,
    breakpoints: [],
    profile
  })

  editor.selected = id
}

export async function loadElf(named: string, elf: ArrayBuffer) {
  const value = await disassembleElf(named, elf)

  const lines = value.error ? [value.error] : value.lines

  createTab(named, lines, { kind: 'elf', elf, breakpoints: value.breakpoints })
}

if (!editor.tabs.length) {
  createTab('Untitled', [''])
}
