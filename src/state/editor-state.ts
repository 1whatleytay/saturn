import { reactive, watch } from 'vue'
import { Editor } from '../utils/editor'
import { tab } from './tabs-state'
import { cursor } from './cursor-state'
import { Language, Token } from '../utils/languages/language'
import { MipsHighlighter } from '../utils/languages/mips/language'

export const storage = reactive({
  editor: createEditor(),
  language: createLanguage(),
  highlights: [] as Token[][]
})

export function editor() {
  return storage.editor
}

function highlight(line: number, deleted: number, lines: string[]) {
  const tokens = lines.map(part => storage.language.highlight(part))

  storage.highlights.splice(line, deleted, ...tokens)
}

function handleDirty(line: number, deleted: number, lines: string[]) {
  highlight(line, deleted, lines)

  const current = tab()

  if (current) {
    current.marked = true
  }
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

  if (tab && tab.lines) {
    highlight(0, 0, tab.lines)
  }
})
