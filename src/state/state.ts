import { useSettings } from '../utils/settings'
import { useCursor } from '../utils/cursor'
import { SelectionIndex } from '../utils/editor'
import { useHighlights } from '../utils/highlights'
import { regular } from '../utils/query/text-size'
import { useStorage } from '../utils/storage'
import { useFind } from '../utils/find'
import { useSuggestions } from '../utils/suggestions'
import { tab } from './tabs-state'

const settings = useSettings()

export const find = useFind()

export const highlights = useHighlights(
  text => regular.calculate(text).width
)

export const { storage } = useStorage(highlights)

export const suggestions = useSuggestions(() => storage.language)

function showSuggestionsAt(cursor: SelectionIndex) {
  suggestions.showSuggestions(storage.highlights[cursor.line], cursor.index)
}

function dismissFindState() {
  find.state.show = false
}

export const {
  position,
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
  settings,
  suggestions,
  showSuggestionsAt,
  dismissFindState
)
