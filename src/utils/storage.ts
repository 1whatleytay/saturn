import { reactive, watch } from 'vue'
import { Editor } from './editor'
import { collectLines, EditorTab } from './tabs'
import { Language, Token } from './languages/language'
import { MipsHighlighter } from './languages/mips/language'
import { assembleText } from './mips'
import { HighlightsInterface } from './highlights'
import { SuggestionsStorage } from './languages/suggestions'

export interface StorageState {
  editor: Editor
  language: Language
  highlights: Token[][]
  debounce: number | null
}

export interface StorageInterface {
  suggestionsStorage(): SuggestionsStorage
}

export type StorageResult = StorageInterface & {
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
    language: createLanguage(),
    highlights: [] as Token[][],
    debounce: null as number | null
  } as StorageState)

  // Not reactive.
  let suggestions = new SuggestionsStorage()

  function highlight(line: number, deleted: number, lines: string[]) {
    const result = lines.map(part => storage.language.highlight(part))

    storage.highlights.splice(line, deleted, ...result.map(x => x.tokens))
    suggestions.update(line, deleted, result.map(x => x.suggestions))
  }

  async function checkSyntax() {
    const result = await assembleText(collectLines(tab()?.lines ?? []))

    if (result.status === 'Error' && result.marker) {
      const tokens = storage.highlights[result.marker.line]

      error.setHighlight(result.marker.line, tokens, result.marker.offset, result.message)
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
      current?.lines ?? ['Nothing yet.'],
      current?.cursor ?? { line: 0, index: 0 },
      handleDirty,
      (current?.writable ?? false) ? undefined : () => false // weird
    )
  }

  function createLanguage(): Language {
    return new MipsHighlighter()
  }

  function highlightAll(current: EditorTab | null) {
    // Might need to look at tab file extension to pick languages
    storage.editor = createEditor()
    storage.language = createLanguage()
    storage.highlights = [] // needs highlighting here

    suggestions = new SuggestionsStorage()

    dispatchCheckSyntax()

    error.dismissHighlight()

    if (current && current.lines) {
      highlight(0, 0, current.lines)
    }
  }

  highlightAll(tab())
  watch(() => tab(), tab => { highlightAll(tab) })

  return {
    storage,
    suggestionsStorage: () => suggestions
  } as StorageResult
}
