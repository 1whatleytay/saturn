import { computed, reactive } from 'vue'

import { tab } from './tabs-state'

import { regular } from '../utils/text-size'
import { consumeBackwards, consumeForwards } from '../utils/alt-consume'
import { hasActionKey } from '../utils/shortcut-key'
import { settings } from './settings-state'
import { SelectionIndex, SelectionRange } from '../utils/editor'
import { editor, storage } from './editor-state'
import { useSuggestions } from '../utils/suggestions'

export type Cursor = SelectionIndex & {
  offsetX: number
  offsetY: number
}

export const cursor = reactive({
  line: 0,
  index: 0,
  offsetX: 0,
  offsetY: 0,

  highlight: null
} as Cursor & {
  highlight: Cursor | null
  pressedBackspace: boolean
})

export const {
  showSuggestions,
  dismissSuggestions,
  moveIndex,
  suggestions,
  mergeSuggestion,
  hasSuggestions,
  flushSuggestions
} = useSuggestions(storage.language)

function showCursorSuggestions() {
  showSuggestions(storage.highlights[cursor.line], cursor.index)
}

export function selectionRange(): SelectionRange | null {
  if (!cursor.highlight) {
     return null
  }

  // Took out technical debt here and the methods in EditorBody for selection.
  const highlightBeforeLine = cursor.highlight.line < cursor.line
  const highlightBeforeIndex = cursor.highlight.line === cursor.line
    && cursor.highlight.index < cursor.index

  if (highlightBeforeLine || highlightBeforeIndex) {
    // assert cursor.highlight.line <= cursor.line
    return {
      startLine: cursor.highlight.line,
      startIndex: cursor.highlight.index,
      endLine: cursor.line,
      endIndex: cursor.index
    }
  } else {
    // assert cursor.highlight.line >= cursor.line
    return {
      startLine: cursor.line,
      startIndex: cursor.index,
      endLine: cursor.highlight.line,
      endIndex: cursor.highlight.index
    }
  }
}

export function makeSelection() {
  if (!cursor.highlight) {
    cursor.highlight = {
      line: cursor.line,
      index: cursor.index,
      offsetX: cursor.offsetX,
      offsetY: cursor.offsetY
    }
  }
}

export function clearSelection() {
  cursor.highlight = null
}

export function setSelection(selection: boolean) {
  if (selection) {
    makeSelection()
  } else {
    clearSelection()
  }
}

export function getSelection(): string | null {
  const range = selectionRange()

  if (!range) {
    return null
  }

  return editor().grab(range)
}

export function dropSelection(): boolean {
  const range = selectionRange()

  if (!range) {
    return false
  }

  editor().drop(range)

  clearSelection()
  putCursor({ line: range.startLine, index: range.startIndex })

  return true
}

function cursorPosition(index: SelectionIndex): Cursor {
  let underflow = false
  let overflow = false

  let actualLine = index.line
  const count = editor().lineCount()

  if (actualLine >= count) {
    overflow = true
    actualLine = count - 1
  } else if (actualLine < 0) {
    underflow = true
    actualLine = 0
  }

  const text = editor().lineAt(actualLine)

  let actualIndex: number

  if (underflow) {
    actualIndex = 0
  } else if (overflow) {
    actualIndex = text.length
  } else {
    actualIndex = Math.max(index.index, 0)
  }

  const leading = text.substring(0, actualIndex)
  const size = regular.calculate(leading)

  return {
    line: actualLine,
    index: actualIndex,
    offsetX: size.width,
    offsetY: size.height * actualLine
  }
}

export function putCursor(index: SelectionIndex, set: Cursor = cursor) {
  const position = cursorPosition(index)

  set.line = position.line
  set.index = position.index
  set.offsetX = position.offsetX
  set.offsetY = position.offsetY
}

function moveLeft(alt: boolean = false, shift: boolean = false) {
  const text = editor().lineAt(cursor.line)

  const consume = alt
    ? consumeBackwards(text, cursor.index)
    : 1

  let line = cursor.line
  let move = cursor.index - consume

  const range = selectionRange()

  if (!shift && range) {
    putCursor({ line: range.startLine, index: range.startIndex })
    clearSelection()

    return
  }

  if (shift && !range) {
    makeSelection()
  }

  if (move < 0 && line > 0) {
    line -= 1

    move = editor().lineAt(line).length
  }

  putCursor({ line, index: move })

  if (hasSuggestions()) {
    showCursorSuggestions()
  }
}

function moveRight(alt: boolean = false, shift: boolean = false) {
  const text = editor().lineAt(cursor.line)

  const consume = alt ? consumeForwards(text, cursor.index) : 1

  let line = cursor.line
  let move = cursor.index + consume

  const range = selectionRange()

  if (!shift && range) {
    putCursor({ line: range.endLine, index: range.endIndex })
    clearSelection()

    return
  }

  if (shift && !range) {
    makeSelection()
  }

  if (text.length < move) {
    line += 1
    move = 0
  }

  putCursor({ line, index: move })

  if (hasSuggestions()) {
    showCursorSuggestions()
  }
}

function moveDown(shift: boolean = false) {
  if (hasSuggestions()) {
    console.log('moving down')
    return moveIndex(+1)
  }

  setSelection(shift)

  putCursor({ line: cursor.line + 1, index: cursor.index })
}

function moveUp(shift: boolean = false) {
  if (hasSuggestions()) {
    return moveIndex(-1)
  }

  setSelection(shift)

  putCursor({ line: cursor.line - 1, index: cursor.index })
}

function hitTab(shift: boolean = false) {
  const region = selectionRange()

  const adjustCursor = (line: number, alignment: number) => {
    if (line === cursor.line) {
      putCursor({ line: cursor.line, index: cursor.index + alignment })
    }

    if (cursor.highlight && line == cursor.highlight.line) {
      putCursor({ line: cursor.highlight.line, index: cursor.highlight.index + alignment }, cursor.highlight)
    }
  }

  if (shift) {
    if (region) {
      for (let line = region.startLine; line <= region.endLine; line++) {
        const alignment = editor().dropTab(line, settings.tabSize)

        adjustCursor(line, -alignment)
      }
    } else {
      const alignment = editor().dropTab(cursor.line, settings.tabSize)

      putCursor({ line: cursor.line, index: cursor.index - alignment })
    }
  } else {
    const alignment = settings.tabSize
    const tabs = ' '.repeat(alignment)

    if (region) {
      for (let line = region.startLine; line <= region.endLine; line++) {
        editor().put({ line, index: 0 }, tabs)

        adjustCursor(line, +alignment)
      }
    } else {
      editor().put(cursor, tabs)

      putCursor({ line: cursor.line, index: cursor.index + alignment })
    }
  }
}

export const tabBody = computed(() => tab()?.lines ?? ['Nothing yet.'])

export function pasteText(text: string) {
  dropSelection()
  
  putCursor(editor().paste(cursor, text))
}

function handleActionKey(event: KeyboardEvent) {
  // assert hasActionKey(event)

  switch (event.key) {
    case 'a':
      const count = editor().lineCount()

      if (count > 0) {
        const end = count - 1
        const text = editor().lineAt(end)

        putCursor({ line: 0, index: 0 }, cursor)
        cursor.highlight = cursorPosition({ line: end, index: text.length })
      }

      break

    case 'z': {
      const value = editor().undo()
      
      if (value) {
        putCursor(value)
        cursor.highlight = null
      }
      
      break
    }
  }
}

export function handleKey(event: KeyboardEvent) {
  const last = cursor.pressedBackspace
  cursor.pressedBackspace = false

  switch (event.key) {
    case 'ArrowLeft':
      moveLeft(event.altKey, event.shiftKey)
      break

    case 'ArrowRight':
      moveRight(event.altKey, event.shiftKey)
      break

    case 'ArrowDown':
      moveDown(event.shiftKey)
      break

    case 'ArrowUp':
      moveUp(event.shiftKey)
      break

    case 'Escape':
      dismissSuggestions()
      event.preventDefault()
      break

    case 'Tab':
      hitTab(event.shiftKey)
      event.preventDefault()
      break

    case 'Backspace':
      if (!last) {
        editor().commit()
      }

      cursor.pressedBackspace = true

      if (!dropSelection()) {
        putCursor(editor().backspace(cursor, event.altKey))
      }

      if (hasSuggestions()) {
        showCursorSuggestions()
      }

      break

    case 'Enter':
      if (event.shiftKey) {
        putCursor({
          line: cursor.line,
          index: editor().lineAt(cursor.line).length
        })

        dismissSuggestions()
      } else if (flushSuggestions()) {
        const suggestion = mergeSuggestion()

        if (suggestion) {
          editor().drop({
            startLine: cursor.line,
            endLine: cursor.line,
            startIndex: suggestion.start,
            endIndex: suggestion.start + suggestion.remove
          })

          putCursor(editor().paste({
            line: cursor.line,
            index: suggestion.start
          }, suggestion.insert))

          editor().commit()

          dismissSuggestions()

          return
        }
      }

      editor().commit()

      dropSelection()
      putCursor(editor().newline(cursor))

      break

    default:
      if (event.metaKey || event.ctrlKey) {
        // Nested if here...
        if (hasActionKey(event)) {
           handleActionKey(event)
        }
        /* handle meta */
      } else if (event.key.length === 1) {
        dropSelection()
        putCursor(editor().put(cursor, event.key))

        showCursorSuggestions()
      }

      break
  }
}

const defaultCursorPush = 4
function putCursorAtCoordinates(x: number, y: number) {
  const count = editor().lineCount()

  if (count <= 0) {
    return
  }

  const { height } = regular.calculate('')

  const lineIndex = Math.floor(y / height)
  const line = Math.min(Math.max(lineIndex, 0), count - 1)
  const text = editor().lineAt(line)

  const index = regular.position(text, x, defaultCursorPush)

  putCursor({ line, index })
}

export function dropCursor(x: number, y: number) {
  cursor.highlight = null
  editor().commit()
  dismissSuggestions()

  putCursorAtCoordinates(x, y)
}

export function dragTo(x: number, y: number) {
  makeSelection()
  putCursorAtCoordinates(x, y)

  if (cursor.highlight
    && cursor.highlight.line === cursor.line
    && cursor.highlight.index === cursor.index) {
    cursor.highlight = null
  }
}

export function lineStart(line: number): number {
  const { height } = regular.calculate('')

  return line * height
}
