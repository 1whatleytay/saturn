import { computed, ComputedRef, reactive, watch } from 'vue'
import { Editor } from './editor'
import { collectLines, EditorTab } from './tabs'
import { Language, Token } from './languages/language'
import { MipsHighlighter } from './languages/mips/language'
import { HighlightsInterface } from './highlights'
import { SuggestionsStorage } from './languages/suggestions'
import { backend } from '../state/backend'

export interface StorageState {
  // editor: Editor
  highlights: Token[][]
  debounce: number | null
}

export interface StorageInterface {
  suggestionsStorage(): SuggestionsStorage
}

export type StorageResult = StorageInterface & {
  editor: ComputedRef<Editor>
  storage: StorageState
}

type ShiftCallback = (line: number, deleted: number, insert: string[]) => void

export function useStorage(
  error: HighlightsInterface,
  tab: () => EditorTab | null,
  dirty: ShiftCallback = () => {}
): StorageResult {
  const storage = reactive({
    editor: createEditor(),
    highlights: [] as Token[][],
    debounce: null as number | null,
  } as StorageState)

  // Not reactive.
  let suggestions = new SuggestionsStorage()

  function highlight(line: number, deleted: number, lines: string[]) {
    const result = lines.map((part) => storage.language.highlight(part))

    storage.highlights.splice(line, deleted, ...result.map((x) => x.tokens))
    suggestions.update(
      line,
      deleted,
      result.map((x) => x.suggestions)
    )
  }

  async function checkSyntax() {
    const current = tab()

    const result = await backend.assembleText(current?.state.doc.toString() ?? '', current?.path ?? null)

    if (result.status === 'Error' && result.marker) {
      const tokens = storage.highlights[result.marker.line]

      error.setHighlight(
        result.marker.line,
        result.marker.offset,
        result.message
      )
    } else {
      error.dismissHighlight()
    }
  }

  function dispatchCheckSyntax() {
    if (tab()?.profile?.kind === 'asm') {
      if (storage.debounce) {
        window.clearTimeout(storage.debounce)
      }

      storage.debounce = window.setTimeout(checkSyntax, 500)
    }
  }

  function shiftBreakpoints(line: number, deleted: number, replaced: number) {
    const current = tab()

    if (!current) {
      return
    }

    const breakpoints = current.breakpoints

    let moved = false

    for (let a = 0; a < breakpoints.length; a++) {
      const bp = breakpoints[a]

      if (bp < line + Math.min(deleted, replaced)) {
      } else if (deleted > replaced && bp < line + deleted) {
        breakpoints.splice(a, 1)
        a -= 1
      } else {
        breakpoints[a] += replaced - deleted
        moved = true
      }
    }

    if (moved) {
      current.breakpoints = [...new Set(current.breakpoints)]
    }
  }

  function handleDirty(line: number, deleted: number, lines: string[]) {
    highlight(line, deleted, lines)

    const current = tab()

    if (current) {
      current.marked = true
    }

    error.shiftHighlight(line, deleted, lines.length)
    shiftBreakpoints(line, deleted, lines.length)

    dirty(line, deleted, lines)

    dispatchCheckSyntax()
  }

  function createEditor(): Editor {
    const current = tab()

    return new Editor(
      current?.state.doc.toString().split("\n") ?? ['Nothing yet.'],
      { line: 0, index: 0 },
      handleDirty,
      current?.writable ?? false ? undefined : () => false // weird
    )
  }

  const editor = computed(() => {
    return createEditor()
  })

  function highlightAll() {
    // Might need to look at tab file extension to pick languages
    // storage.editor = createEditor()
    storage.highlights = [] // needs highlighting here


    dispatchCheckSyntax()

    error.dismissHighlight()

  }

  watch(
    () => tab()?.doc,
    (tab) => highlightAll(),
    {immediate: true}
  )

  return {
    editor,
    storage,
    suggestionsStorage: () => suggestions,
  } as StorageResult
}
