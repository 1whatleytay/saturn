import { useSettings } from '../utils/settings'
import { useCursor } from '../utils/cursor'
import { SelectionIndex } from '../utils/editor'
import { useHighlights } from '../utils/highlights'
import { regular } from '../utils/query/text-size'
import { useStorage } from '../utils/storage'
import { useFind } from '../utils/find'
import { useSuggestions } from '../utils/suggestions'
import { useTabs } from '../utils/tabs'
import { GotoMessage, useGoto } from '../utils/goto'

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

export const highlights = useHighlights(widthQuery)
export const gotoHighlights = useHighlights<GotoMessage>(widthQuery)

function onDirty(line: number, deleted: number, insert: string[]) {
  find.dirty(line, deleted, insert)
  gotoHighlights.shiftHighlight(line, deleted, insert.length)
}

const storageResult = useStorage(highlights, tab, onDirty)

export const { editor, storage, suggestionsStorage } = storageResult

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
  () => tab()?.cursor ?? { line: 0, index: 0, highlight: null },
  settings.editor,
  regular,
  24,
  suggestions,
  showSuggestionsAt
)
