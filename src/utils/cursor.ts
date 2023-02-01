import { Editor, LineRange, SelectionIndex } from './editor'
import { reactive } from 'vue'
import { MergeSuggestion, SuggestionsInterface } from './suggestions'
import { regular } from './query/text-size'
import { consumeBackwards, consumeForwards } from './query/alt-consume'
import { hasActionKey } from './query/shortcut-key'
import { selectionRange, CursorState } from '../state/tabs-state'
import { Settings } from './settings'
import { grabWhitespace } from './languages/language'

export interface CursorPosition {
  offsetX: number
  offsetY: number
}

export type CursorPositionState = CursorPosition & { highlight: CursorPosition | null }

export interface CursorInterface {
  jump(index: SelectionIndex): void
  getSelection(): string | null
  dropSelection(): void
  pasteText(text: string): void
  dropCursor(x: number, y: number): void
  dragTo(x: number, y: number): void
  lineStart(line: number): number
  handleKey(event: KeyboardEvent): void
  applyMergeSuggestion(suggestion: MergeSuggestion): void
}
export type CursorResult = CursorInterface & {
  position: CursorPositionState
}

export function useCursor(
  editor: () => Editor,
  cursor: () => CursorState,
  settings: Settings,
  suggestions: SuggestionsInterface,
  showSuggestionsAt: (cursor: SelectionIndex) => void
): CursorResult {
  const position = reactive({
    offsetX: 0,
    offsetY: 0,

    highlight: null
  } as CursorPositionState)

  let pressedBackspace = false

  function applyMergeSuggestion(suggestion: MergeSuggestion) {
    editor().drop({
      startLine: cursor().line,
      endLine: cursor().line,
      startIndex: suggestion.start,
      endIndex: suggestion.start + suggestion.remove
    })

    putCursor(editor().paste({
      line: cursor().line,
      index: suggestion.start
    }, suggestion.insert))

    editor().commit()

    suggestions.dismissSuggestions()
  }

  function makeSelection() {
    const value = cursor()

    if (!value.highlight) {
      value.highlight = {
        line: value.line,
        index: value.index,
      }
    }
  }

  function clearSelection() {
    cursor().highlight = null
  }

  function setSelection(selection: boolean) {
    if (selection) {
      makeSelection()
    } else {
      clearSelection()
    }
  }

  function getSelection(): string | null {
    const range = selectionRange(cursor())

    if (!range) {
      return null
    }

    return editor().grab(range)
  }

  function dropSelection(): boolean {
    const range = selectionRange(cursor())

    if (!range) {
      return false
    }

    editor().drop(range)

    clearSelection()
    putCursor({ line: range.startLine, index: range.startIndex })

    return true
  }

  function cursorPosition(index: SelectionIndex): SelectionIndex & CursorPosition {
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

  function putCursor(
    index: SelectionIndex,
    from: SelectionIndex = cursor(),
    pos: CursorPosition = position
  ) {
    const position = cursorPosition(index)

    from.line = position.line
    from.index = position.index
    pos.offsetX = position.offsetX
    pos.offsetY = position.offsetY
  }

  function moveLeft(alt: boolean = false, shift: boolean = false) {
    const value = cursor()

    bringCursorInline()
    const text = editor().lineAt(value.line)

    const consume = alt
      ? consumeBackwards(text, value.index)
      : 1

    let line = value.line
    let move = value.index - consume

    const range = selectionRange(value)

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

    if (suggestions.hasSuggestions()) {
      showSuggestionsAt(cursor())
    }
  }

  function moveRight(alt: boolean = false, shift: boolean = false) {
    const value = cursor()

    bringCursorInline()
    const text = editor().lineAt(value.line)

    const consume = alt ? consumeForwards(text, value.index) : 1

    let line = value.line
    let move = value.index + consume

    const range = selectionRange(value)

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

    if (suggestions.hasSuggestions()) {
      showSuggestionsAt(cursor())
    }
  }

  function moveDown(shift: boolean = false) {
    if (suggestions.hasSuggestions()) {
      return suggestions.moveIndex(+1)
    }

    setSelection(shift)

    putCursor({ line: cursor().line + 1, index: cursor().index })
  }

  function moveUp(shift: boolean = false) {
    if (suggestions.hasSuggestions()) {
      return suggestions.moveIndex(-1)
    }

    setSelection(shift)

    putCursor({ line: cursor().line - 1, index: cursor().index })
  }

  function hitTab(shift: boolean = false) {
    const value = cursor()
    const region = selectionRange(value)

    const adjustCursor = (line: number, alignment: number) => {
      if (line === value.line) {
        putCursor({ line: value.line, index: value.index + alignment })
      }

      if (value.highlight && line == value.highlight.line) {
        position.highlight = { offsetX: 0, offsetY: 0 }

        putCursor({
          line: value.highlight.line,
          index: value.highlight.index + alignment
        }, value.highlight, position.highlight)
      }
    }

    if (shift) {
      if (region) {
        for (let line = region.startLine; line <= region.endLine; line++) {
          const alignment = editor().dropTab(line, settings.tabSize)

          adjustCursor(line, -alignment)
        }
      } else {
        const alignment = editor().dropTab(value.line, settings.tabSize)

        putCursor({ line: value.line, index: value.index - alignment })
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
        editor().put(value, tabs)

        putCursor({ line: value.line, index: value.index + alignment })
      }
    }
  }

  // Spaghetti hits hard...
  function commentTrimSize(text: string): LineRange | 'ignore' | null {
    const { leading } = grabWhitespace(text)

    if (leading.length === text.length) {
      return 'ignore'
    }

    const rest = text.substring(leading.length)

    if (rest.startsWith("# ")) {
      return { start: leading.length, end: leading.length + 2 }
    } else if (rest.startsWith("#")) {
      return { start: leading.length, end: leading.length + 1 }
    }

    return null
  }

  function comment() {
    const value = cursor()
    const range = selectionRange(value)

    let start = range?.startLine ?? value.line
    let end = range?.endLine ?? value.line

    let all = true
    const crops = []

    // This is beyond complicated
    let cursorIndex = value.index
    let highlightIndex = value.highlight?.index

    const reposition = (index: number, range: LineRange): number => {
      if (index < range.start) {
        return index
      } else if (index < range.end) {
        return range.start
      } else {
        return index - (range.end - range.start)
      }
    }

    for (let a = start; a <= end; a++) {
      const size = commentTrimSize(editor().lineAt(a))

      // Empty lines should not affect whether or not we comment.
      if (size === 'ignore') {
        crops.push(null)

        continue
      }

      crops.push(size)

      if (!size) {
        all = false
      } else {
        // could be done via query
        if (value.line === a) {
          cursorIndex = reposition(value.index, size)
        }

        if (value.highlight && value.highlight?.line === a) {
          highlightIndex = reposition(value.highlight.index, size)
        }
      }
    }

    if (all) {
      editor().crop(start, crops)

      putCursor({ line: value.line, index: cursorIndex })

      if (value.highlight && highlightIndex) {
        // Just make it computed, am i right?
        position.highlight = { offsetX: 0, offsetY: 0 }

        putCursor({
          line: value.highlight.line, index: highlightIndex
        }, value.highlight, position.highlight)
      }
    } else {
      editor().prefix(start, end, '# ', true)

      // This should... maybe be correct cursor positioning
      const { leading: cursorLeading } = grabWhitespace(editor().lineAt(value.line))
      if (value.index >= cursorLeading.length) {
        putCursor({ line: value.line, index: value.index + 2 })
      }

      if (value.highlight && position.highlight) {
        const { leading: highlightLeading } = grabWhitespace(editor().lineAt(value.highlight.line))

        if (value.highlight.index >= highlightLeading.length) {
          putCursor({
            line: value.highlight.line, index: value.highlight.index + 2
          }, value.highlight, position.highlight)
        }
      }
    }
  }

  function pasteText(text: string) {
    dropSelection()

    putCursor(editor().paste(cursor(), text))
  }

  function bringCursorInline(index: SelectionIndex = cursor()) {
    const line = editor().lineAt(index.line)

    if (index.index > line.length) {
      index.index = line.length
    }
  }

  function handleActionKey(event: KeyboardEvent) {
    // assert hasActionKey(event)

    const value = cursor()

    switch (event.key) {
      case 'a':
        const count = editor().lineCount()

        if (count > 0) {
          const end = count - 1
          const text = editor().lineAt(end)

          putCursor({ line: 0, index: 0 })
          value.highlight = cursorPosition({ line: end, index: text.length })
        }

        break

      case '/':
        comment()
        break

      case 'z': {
        const frame = editor().undo()

        if (frame) {
          putCursor(frame)
          value.highlight = null
        }

        break
      }
    }
  }

  function handleKey(event: KeyboardEvent) {
    const value = cursor()

    const last = pressedBackspace
    pressedBackspace = false

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
        if (suggestions.hasSuggestions()) {
          suggestions.dismissSuggestions()
        }

        event.preventDefault()

        break

      case 'Tab':
        hitTab(event.shiftKey)
        event.preventDefault()
        break

      case 'Backspace':
      case 'Delete':
        const doDelete = event.key === 'Delete'

        if (!last) {
          editor().commit()
        }

        pressedBackspace = true

        if (!dropSelection()) {
          bringCursorInline()
          let nextPosition: SelectionIndex

          if (doDelete) {
            nextPosition = editor().deleteForwards(value, event.altKey)
          } else {
            nextPosition = editor().backspace(value, event.altKey)
          }

          putCursor(nextPosition)
        }

        if (suggestions.hasSuggestions()) {
          showSuggestionsAt(cursor())
        }

        break

      case 'Enter':
        if (event.shiftKey) {
          putCursor({
            line: value.line,
            index: editor().lineAt(value.line).length
          })

          suggestions.dismissSuggestions()
        } else if (suggestions.flushSuggestions()) {
          const suggestion = suggestions.mergeSuggestion()

          if (suggestion) {
            return applyMergeSuggestion(suggestion)
          }
        }

        editor().commit()

        dropSelection()
        putCursor(editor().newline(value))

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
          putCursor(editor().put(value, event.key))

          showSuggestionsAt(cursor())
        }

        break
    }
  }

  const defaultCursorPush = 0
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

  function dropCursor(x: number, y: number) {
    cursor().highlight = null
    editor().commit()
    suggestions.dismissSuggestions()

    putCursorAtCoordinates(x, y)
  }

  function dragTo(x: number, y: number) {
    const value = cursor()

    makeSelection()
    putCursorAtCoordinates(x, y)

    if (value.highlight
      && value.highlight.line === value.line
      && value.highlight.index === value.index) {
      value.highlight = null
    }
  }

  function jump(index: SelectionIndex) {
    putCursor(index)

    cursor().highlight = null
    position.highlight = null
  }

  function lineStart(line: number): number {
    const { height } = regular.calculate('')

    return line * height
  }

  return {
    position,
    jump,
    lineStart,
    getSelection,
    dropSelection,
    dropCursor,
    dragTo,
    pasteText,
    handleKey,
    applyMergeSuggestion
  }
}
