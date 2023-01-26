import { reactive, watch } from 'vue'
import { Editor } from '../utils/editor'
import { collectLines, tab } from './tabs-state'
import { cursor } from './cursor-state'
import { Language, Token } from '../utils/languages/language'
import { MipsHighlighter } from '../utils/languages/mips/language'
import { useErrorHighlight } from '../utils/error-highlight'
import { regular } from '../utils/text-size'
import { assembleText } from '../utils/mips'

export const storage = reactive({
  editor: createEditor(),
  language: createLanguage(),
  highlights: [] as Token[][],
  debounce: null as number | null
})

export const {
  state: errorState,
  setHighlight
} = useErrorHighlight(
  text => regular.calculate(text).width,
  line => storage.highlights[line]
)

export function editor() {
  return storage.editor
}

function highlight(line: number, deleted: number, lines: string[]) {
  const tokens = lines.map(part => storage.language.highlight(part))

  storage.highlights.splice(line, deleted, ...tokens)
}

async function checkSyntax() {
  const result = await assembleText(collectLines(tab()?.lines ?? []))

  if (result.status === 'Error' && result.marker) {
    setHighlight(result.marker.line, result.marker.offset, result.message)
  } else {
    errorState.highlight = null
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

  if (errorState.highlight) {
    if (line <= errorState.highlight.line && errorState.highlight.line < line + deleted) {
      errorState.highlight = null
    } else if (lines.length !== deleted && errorState.highlight.line >= line + deleted) {
      errorState.highlight.line += lines.length - deleted
    }

    // check breakpoints too
  }

  dispatchCheckSyntax()
}

function createEditor(): Editor {
  return new Editor(
    tab()?.lines ?? ['Nothing yet.'],
    () => cursor, // This is cyclic, but I can't bring myself to care.
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

  errorState.highlight = null

  if (tab && tab.lines) {
    highlight(0, 0, tab.lines)
  }
})
