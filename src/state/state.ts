import { useSettings } from '../utils/settings'
import { useCursor } from '../utils/cursor'
import { SelectionIndex } from '../utils/editor'
import { useHighlights } from '../utils/highlights'
import { regular } from '../utils/query/text-size'
import { useStorage } from '../utils/storage'
import { useFind } from '../utils/find'
import { useSuggestions } from '../utils/suggestions'
import { CursorState, useTabs } from '../utils/tabs'
import { GotoMessage, useGoto } from '../utils/goto'
import { useSymbolHighlight } from '../utils/symbol-highlight'
import { ref, watch } from 'vue'
import { InstructionLine } from '../utils/mips/mips'

export const settings = useSettings()

function widthQuery(text: string) {
  return regular.calculate(text).width
}

export const {
  tabsState,
  tab,
  tabBody,
  createTab,
  closeTab,
  loadElf,
  saveModal,
  showSettings
} = useTabs()

export const find = useFind(() => tabBody.value, widthQuery)

export const errorHighlights = useHighlights(widthQuery)
export const gotoHighlights = useHighlights<GotoMessage>(widthQuery)

function cursorState(): CursorState {
  return tab()?.cursor ?? { line: 0, index: 0, highlight: null }
}

function cursorIndex(): SelectionIndex {
  const state = cursorState()

  return { line: state.line, index: state.index }
}


const storageResult = useStorage(errorHighlights, tab, onDirty)

export const { editor, storage, suggestionsStorage } = storageResult

export const {
  updateCursor: updateCursorSymbol,
  highlights: symbolHighlights
} = useSymbolHighlight(storageResult, widthQuery)

function onDirty(line: number, deleted: number, insert: string[]) {
  find.dirty(line, deleted, insert)
  gotoHighlights.shiftHighlight(line, deleted, insert.length)
  updateCursorSymbol(cursorState())
}

watch(() => {
  const cursor = cursorIndex()
  const line = tab()?.lines[cursor.line]

  const index = line ? Math.min(line.length, cursor.index) : cursor.index

  return { line: cursor.line, index }
}, updateCursorSymbol)

export const goto = useGoto(gotoHighlights, storageResult)

export const suggestions = useSuggestions(
  () => storage.language,
  suggestionsStorage
)

function showSuggestionsAt(cursor: SelectionIndex) {
  const highlights = storage.highlights[cursor.line]

  if (highlights) {
    suggestions.showSuggestions(storage.highlights[cursor.line], cursor.index)
  }
}

export const showExportRegionsDialog = ref(false)

export const {
  range,
  position,
  jump,
  lineStart,
  getSelection,
  dropSelection,
  pasteText,
  dropCursor,
  dragTo,
  cursorCoordinates,
  handleKey,
  applyMergeSuggestion,
} = useCursor(
  () => editor.value,
  cursorState,
  settings.editor,
  regular,
  24,
  suggestions,
  showSuggestionsAt
)

export const buildLines = ref(null as InstructionLine[] | null)
