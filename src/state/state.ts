import { useSettings } from '../utils/settings'
import { useCursor } from '../utils/cursor'
import { SelectionIndex } from '../utils/editor'
import { useHighlights } from '../utils/highlights'
import { regular } from '../utils/query/text-size'
import { useStorage } from '../utils/storage'
import { useFind } from '../utils/find'
import { useSuggestions } from '../utils/suggestions'
import { useTabs } from '../utils/tabs'

export const settings = useSettings()

function widthQuery(text: string) {
  return regular.calculate(text).width
}

export const {
  editor,
  tab,
  tabBody,
  createTab,
  closeTab,
  loadElf
} = useTabs()

export const find = useFind(() => tabBody.value, widthQuery)

export const highlights = useHighlights(widthQuery)

export const { storage } = useStorage(highlights, find, tab)

export const suggestions = useSuggestions(() => storage.language)

function showSuggestionsAt(cursor: SelectionIndex) {
  suggestions.showSuggestions(storage.highlights[cursor.line], cursor.index)
}

export const {
  position,
  jump,
  lineStart,
  getSelection,
  dropSelection,
  pasteText,
  dropCursor,
  dragTo,
  handleKey,
  applyMergeSuggestion
} = useCursor(
  () => storage.editor,
  () => tab()?.cursor ?? { line: 0, index: 0, highlight: null },
  regular,
  settings,
  suggestions,
  showSuggestionsAt
)
