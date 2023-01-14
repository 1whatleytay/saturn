import { computed, reactive, watch } from 'vue'

import { tab } from '../state/editor-state'

import { regular } from './text-size'
import { consumeBackwards, consumeForwards } from './alt-consume'
import { hasActionKey } from './shortcut-key'
import { settings } from '../state/settings-state'
import { Editor, SelectionIndex, SelectionRange } from './editor'

export type Cursor = SelectionIndex & {
  offsetX: number
  offsetY: number
}

export const cursor = reactive({
  line: 0,
  index: 0,
  offsetX: 0,
  offsetY: 0,

  editor: createEditor(),
  highlight: null
} as Cursor & {
  editor: Editor
  highlight: Cursor | null
  pressedBackspace: boolean
})

function createEditor(): Editor {
  return new Editor(
    tab()?.lines ?? ['Nothing yet.'],
    () => cursor
  )
}

watch(() => tab(), () => {
  cursor.editor = createEditor()
})

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

  return cursor.editor.grab(range)
}

export function dropSelection(): boolean {
  const range = selectionRange()

  if (!range) {
    return false
  }

  cursor.editor.drop(range)

  clearSelection()
  putCursor({ line: range.startLine, index: range.startIndex })

  return true
}

function cursorPosition(index: SelectionIndex): Cursor {
  let underflow = false
  let overflow = false

  let actualLine = index.line
  const count = cursor.editor.lineCount()

  if (actualLine >= count) {
    overflow = true
    actualLine = count - 1
  } else if (actualLine < 0) {
    underflow = true
    actualLine = 0
  }

  const text = cursor.editor.lineAt(actualLine)

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
  const text = cursor.editor.lineAt(cursor.line)

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

    move = cursor.editor.lineAt(line).length
  }

  putCursor({ line, index: move })
}

function moveRight(alt: boolean = false, shift: boolean = false) {
  const text = cursor.editor.lineAt(cursor.line)

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
}

function moveDown(shift: boolean = false) {
  setSelection(shift)

  putCursor({ line: cursor.line + 1, index: cursor.index })
}

function moveUp(shift: boolean = false) {
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
        const alignment = cursor.editor.dropTab(line, settings.tabSize)

        adjustCursor(line, -alignment)
      }
    } else {
      const alignment = cursor.editor.dropTab(cursor.line, settings.tabSize)

      putCursor({ line: cursor.line, index: cursor.index - alignment })
    }
  } else {
    const alignment = settings.tabSize
    const tabs = ' '.repeat(alignment)

    if (region) {
      for (let line = region.startLine; line <= region.endLine; line++) {
        cursor.editor.put({ line, index: 0 }, tabs)

        adjustCursor(line, +alignment)
      }
    } else {
      cursor.editor.put(cursor, tabs)

      putCursor({ line: cursor.line, index: cursor.index + alignment })
    }
  }
}

export const tabBody = computed(() => tab()?.lines ?? ['Nothing yet.'])

export function pasteText(text: string) {
  putCursor(cursor.editor.paste(cursor, text))
}

function handleActionKey(event: KeyboardEvent) {
  // assert hasActionKey(event)

  switch (event.key) {
    case 'a':
      const count = cursor.editor.lineCount()

      if (count > 0) {
        const end = count - 1
        const text = cursor.editor.lineAt(end)

        putCursor({ line: 0, index: 0 }, cursor)
        cursor.highlight = cursorPosition({ line: end, index: text.length })
      }

      break

    case 'z': {
      const frame = cursor.editor.undo()

      if (!frame) {
        return
      }

      const { data, cursor: value } = frame

      const current = tab()

      if (current) {
        // Please work...
        console.log('spliced')
        current.lines.splice(0, current.lines.length, ...data)
      }

      putCursor(value)

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

    case 'Tab':
      hitTab(event.shiftKey)
      event.preventDefault()
      break

    case 'Backspace':
      if (!last) {
        cursor.editor.commit()
      }

      cursor.pressedBackspace = true

      if (!dropSelection()) {
        putCursor(cursor.editor.backspace(cursor, event.altKey))
      }

      break

    case 'Enter':
      cursor.editor.commit()

      dropSelection()
      putCursor(cursor.editor.newline(cursor))

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
        putCursor(cursor.editor.put(cursor, event.key))
      }

      break
  }
}

const defaultCursorPush = 4
function putCursorAtCoordinates(x: number, y: number) {
  const count = cursor.editor.lineCount()

  if (count <= 0) {
    return
  }

  const { height } = regular.calculate('')

  const lineIndex = Math.floor(y / height)
  const line = Math.min(Math.max(lineIndex, 0), count - 1)
  const text = cursor.editor.lineAt(line)

  const index = regular.position(text, x, defaultCursorPush)

  putCursor({ line, index })
}

export function dropCursor(x: number, y: number) {
  cursor.highlight = null
  cursor.editor.commit()

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
