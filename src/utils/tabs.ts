import { computed, ComputedRef, reactive } from 'vue'

import { v4 as uuid } from 'uuid'
import { AssemblyExecutionProfile, disassembleElf, ElfExecutionProfile, ExecutionProfile } from './mips'
import { SelectionIndex, SelectionRange } from './editor'
import { PromptType, saveTab } from './events'
import { SaveModalResult, useSaveModal } from './save-modal'

export type CursorState = SelectionIndex & {
  highlight: SelectionIndex | null
}

export function selectionRange(cursor: CursorState): SelectionRange | null {
  if (!cursor.highlight) {
    return null
  }

  // Took out technical debt here and the methods in EditorBody for selection.
  const highlightBeforeLine = cursor.highlight.line < cursor.line
  const highlightBeforeIndex = cursor.highlight.line === cursor.line
    && cursor.highlight.index < cursor.index

  if (highlightBeforeLine || highlightBeforeIndex) {
    // assert cursor.highlight.line <= cursor.line
    return {
      startLine: cursor.highlight.line,
      startIndex: cursor.highlight.index,
      endLine: cursor.line,
      endIndex: cursor.index
    }
  } else {
    // assert cursor.highlight.line >= cursor.line
    return {
      startLine: cursor.line,
      startIndex: cursor.index,
      endLine: cursor.highlight.line,
      endIndex: cursor.highlight.index
    }
  }
}

export interface EditorTab {
  uuid: string
  title: string
  lines: string[]
  cursor: CursorState
  breakpoints: number[]
  path: string | null
  writable: boolean
  marked: boolean // needs saving
  profile: ExecutionProfile | null
}

export function collectLines(lines: string[]): string {
  return lines.join('\n')
}

export interface Tabs {
  tabs: EditorTab[]
  selected: string | null
}

export interface TabsInterface {
  tab(): EditorTab | null
  closeTab(uuid: string): boolean
  createTab(
    named: string,
    content: string[],
    path?: string,
    profile?: ExecutionProfile
  ): void
  loadElf(named: string, elf: ArrayBuffer): Promise<void>
}

export type TabsResult = TabsInterface & {
  editor: Tabs
  tabBody: ComputedRef<string[]>
  saveModal: SaveModalResult
}

export function useTabs(): TabsResult {
  const editor = reactive({
    tabs: [],
    selected: null,
    execution: null,
    debug: null
  } as Tabs)

  function tab(): EditorTab | null {
    if (editor.selected) {
      return editor.tabs.find(tab => tab.uuid === editor.selected) ?? null
    }

    return null
  }

  const tabBody = computed(() => tab()?.lines ?? ['Nothing yet.'])

  async function discardTab(uuid: string) {
    const index = editor.tabs.findIndex(tab => tab.uuid === uuid)

    if (index === undefined) {
      return
    }

    editor.tabs = editor.tabs.filter(tab => tab.uuid !== uuid)

    if (editor.selected === uuid) {
      if (editor.tabs.length <= 0) {
        editor.selected = null

        return
      }

      const point = Math.min(Math.max(index - 1, 0), editor.tabs.length - 1)

      editor.selected = editor.tabs[point].uuid
    }
  }

  const saveModal = useSaveModal(
    tab => saveTab(tab, PromptType.PromptWhenNeeded),
    tab => discardTab(tab.uuid)
  )

  function closeTab(uuid: string): boolean {
    const tab = editor.tabs.find(tab => tab.uuid === uuid)

    if (tab === undefined) {
      return true
    }

    if (tab.marked) {
      saveModal.present(tab)

      return false
    } else {
      discardTab(uuid).then(() => {})

      return true
    }
  }

  function defaultAssemblyProfile(): AssemblyExecutionProfile  {
    return { kind: 'asm' }
  }

  function createTab(
    named: string,
    content: string[],
    path: string | null = null,
    profile: ExecutionProfile | null = defaultAssemblyProfile(),
    writable: boolean = true
  ) {
    const id = uuid()

    editor.tabs.push({
      uuid: id,
      title: named,
      lines: content,
      breakpoints: [],
      cursor: {
        line: 0,
        index: 0,
        highlight: null
      },
      path,
      writable,
      marked: false,
      profile
    })

    editor.selected = id
  }

  async function loadElf(named: string, elf: ArrayBuffer) {
    const value = await disassembleElf(named, elf)

    const lines = value.error ? [value.error] : value.lines
    const profile = {
      kind: 'elf', elf, breakpoints: value.breakpoints
    } as ElfExecutionProfile

    createTab(named, lines, null, profile, false)
  }

  if (!editor.tabs.length) {
    createTab('Untitled', [''])
  }

  return {
    editor,
    tabBody,
    tab,
    closeTab,
    createTab,
    loadElf,
    saveModal
  }
}