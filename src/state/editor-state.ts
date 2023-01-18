import { reactive, watch } from 'vue'
import { Editor } from '../utils/editor'
import { tab } from './tabs-state'
import { cursor } from './cursor-state'
import { Highlighter, Token } from '../utils/highlighter/highlighter'
import { MipsHighlighter } from '../utils/highlighter/mips'

export const storage = reactive({
  editor: createEditor(),
  highlighter: createHighlighter(),
  highlights: [] as Token[][]
})

export function editor() {
  return storage.editor
}

function highlight(line: number, deleted: number, lines: string[]) {
  const tokens = lines.map(part => storage.highlighter.highlight(part))

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

function createHighlighter(): Highlighter {
  return new MipsHighlighter()
}

watch(() => tab(), tab => {
  // Might need to look at tab file extension to pick highlighter
  storage.editor = createEditor()
  storage.highlighter = createHighlighter()
  storage.highlights = [] // needs highlighting here

  if (tab && tab.lines) {
    highlight(0, 0, tab.lines)
  }
})
