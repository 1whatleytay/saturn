import { reactive, watch } from 'vue'
import { Editor } from './editor'
import { collectLines, tab } from '../state/tabs-state'
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

export function useStorage(highlights: HighlightsInterface, find: FindInterface): StorageResult {
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
        clearInterval(storage.debounce)
      }

      storage.debounce = window.setTimeout(checkSyntax, 500)
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

    dispatchCheckSyntax()
  }

  function createEditor(): Editor {
    return new Editor(
      tab()?.lines ?? ['Nothing yet.'],
      tab()?.cursor ?? { line: 0, index: 0 },
      handleDirty
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
