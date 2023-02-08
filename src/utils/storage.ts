import { reactive, watch } from 'vue'
import { Editor } from './editor'
import { collectLines, EditorTab } from './tabs'
import { Language, Token } from './languages/language'
import { MipsHighlighter } from './languages/mips/language'
import { assembleText } from './mips'
import { HighlightsInterface } from './highlights'
import { FindInterface } from './find'

export interface StorageState {
  editor: Editor
  language: Language
  highlights: Token[][]
  debounce: number | null
}

export interface StorageResult {
  storage: StorageState
}

export function useStorage(highlights: HighlightsInterface, find: FindInterface, tab: () => EditorTab | null): StorageResult {
  const storage = reactive({
    editor: createEditor(),
    language: createLanguage(),
    highlights: [] as Token[][],
    debounce: null as number | null
  } as StorageState)

  function highlight(line: number, deleted: number, lines: string[]) {
    const tokens = lines.map(part => storage.language.highlight(part))

    storage.highlights.splice(line, deleted, ...tokens)
  }

  async function checkSyntax() {
    const result = await assembleText(collectLines(tab()?.lines ?? []))

    if (result.status === 'Error' && result.marker) {
      const tokens = storage.highlights[result.marker.line]

      highlights.setHighlight(result.marker.line, tokens, result.marker.offset, result.message)
    } else {
      highlights.dismissHighlight()
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

    find.dirty(line, deleted, lines)
    highlights.shiftHighlight(line, deleted, lines.length)
    shiftBreakpoints(line, deleted, lines.length)

    dispatchCheckSyntax()
  }

  function createEditor(): Editor {
    return new Editor(
      tab()?.lines ?? ['Nothing yet.'],
      tab()?.cursor ?? { line: 0, index: 0 },
      handleDirty,
      tab()?.writable ?? false
    )
  }

  function createLanguage(): Language {
    return new MipsHighlighter()
  }

  watch(() => tab(), tab => {
    // Might need to look at tab file extension to pick languages
    storage.editor = createEditor()
    storage.language = createLanguage()
    storage.highlights = [] // needs highlighting here

    dispatchCheckSyntax()

    highlights.dismissHighlight()

    if (tab && tab.lines) {
      highlight(0, 0, tab.lines)
    }
  })

  return {
    storage
  } as StorageResult
}
