import { Editor, LineRange, SelectionIndex } from './editor'
import { computed, ComputedRef } from 'vue'
import { MergeSuggestion, SuggestionsInterface } from './suggestions'
import { SizeCalculator } from './query/text-size'
import { consumeBackwards, consumeDirection, consumeForwards } from './query/alt-consume'
import { hasActionKey, hasAltKey } from './query/shortcut-key'
import { selectionRange, CursorState } from './tabs'
import { EditorSettings } from './settings'
import { grabWhitespace } from './languages/language'

export interface CursorPosition {
  offsetX: number
  offsetY: number
}

export interface RangeLine {
  leading: string
  body: string
}

export interface RangeSelection {
  ranges: RangeLine[]
  top: number
}

export interface CursorInterface {
  range(start: number, count: number): RangeSelection | null
  jump(index: SelectionIndex): void
  getSelection(): string | null
  dropSelection(): void
  pasteText(text: string): void
  dropCursor(x: number, y: number, detail?: number, shift?: boolean): void
  dragTo(x: number, y: number): void
  cursorCoordinates(x: number, y: number): SelectionIndex
  lineStart(line: number): number
  handleKey(event: KeyboardEvent): void
  applyMergeSuggestion(suggestion: MergeSuggestion): void
}
export type CursorResult = CursorInterface & {
  position: ComputedRef<CursorPosition>
}

export function useCursor(
  editor: () => Editor,
  cursor: () => CursorState,
  settings: EditorSettings,
  calculator: SizeCalculator,
  lineHeight: number = 24, // precompute this
  suggestions?: SuggestionsInterface,
  showSuggestionsAt?: (cursor: SelectionIndex) => void
): CursorResult {
  function toPosition(index: SelectionIndex): CursorPosition {
    const text = editor().lineAt(index.line)

    // No way to watch state here in cursor, so fall back for any forgetful times.
    if (text === undefined) {
      console.error(`Cursor failed to reset: ${index.line} < ${editor().lineCount()}`)
      return { offsetX: 0, offsetY: 0 }
    }

    const leading = text.substring(0, index.index)
    const { width } = calculator.calculate(leading)

    return {
      offsetX: width,
      offsetY: lineHeight * index.line
    }
  }

  const position = computed(() => {
    return toPosition(cursor())
  })

  function range(start: number, count: number): RangeSelection | null {
    const range = selectionRange(cursor())

    if (!range) {
      return null
    }

    function inBounds(line: number): boolean {
      return start <= line && line < start + count
    }

    let top = null as number | null
    const suggestTop = (line: number) => {
      if (top === null) {
        top = line * lineHeight
      }
    }

    if (range.startLine == range.endLine) {
      if (!inBounds(range.startLine)) {
        return null // ?
      }

      const line = editor().lineAt(range.startLine)

      const ranges = [{
        leading: line.substring(0, range.startIndex),
        body: line.substring(range.startIndex, range.endIndex)
      }]

      return { ranges, top: range.startLine * lineHeight }
    }

    const result = [] as RangeLine[]

    if (inBounds(range.startLine)) {
      suggestTop(range.startLine)
      const first = editor().lineAt(range.startLine)

      result.push({
        leading: first.substring(0, range.startIndex),
        body: first.substring(range.startIndex)
      })
    }

    // Just going to hope this intersection thingy works.
    const intersectionStart = Math.max(range.startLine + 1, start)
    const intersectionEnd = Math.min(range.endLine, start + count)

    if (intersectionStart < intersectionEnd) {
      suggestTop(intersectionStart)
    }

    for (let a = intersectionStart; a < intersectionEnd; a++) {
      result.push({
        leading: '',
        body: editor().lineAt(a)
      })
    }

    if (inBounds(range.endLine)) {
      suggestTop(range.endLine)
      const last = editor().lineAt(range.endLine)

      result.push({
        leading: '',
        body: last.substring(0, range.endIndex)
      })
    }

    return { top: top ?? range.startLine * lineHeight, ranges: result }
  }

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

    suggestions?.dismissSuggestions()
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

  function alignCursor(index: SelectionIndex): SelectionIndex {
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

    return {
      line: actualLine,
      index: actualIndex,
    }
  }

  function promptSuggestions() {
    if ((suggestions?.hasSuggestions() ?? false) && showSuggestionsAt) {
      showSuggestionsAt(cursor())
    }
  }

  function putCursor(
    index: SelectionIndex,
    to: SelectionIndex = cursor()
  ) {
    const position = alignCursor(index)

    to.line = position.line
    to.index = position.index
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

    promptSuggestions()
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

    promptSuggestions()
  }

  function moveDown(shift: boolean = false) {
    if (suggestions && suggestions.hasSuggestions()) {
      return suggestions.moveIndex(+1)
    }

    setSelection(shift)

    putCursor({ line: cursor().line + 1, index: cursor().index })
  }

  function moveUp(shift: boolean = false) {
    if (suggestions && suggestions.hasSuggestions()) {
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
        putCursor({
          line: value.highlight.line,
          index: value.highlight.index + alignment
        }, value.highlight)
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

      if (region) {
        const tabs = ' '.repeat(alignment)

        for (let line = region.startLine; line <= region.endLine; line++) {
          editor().put({ line, index: 0 }, tabs)

          adjustCursor(line, +alignment)
        }
      } else {
        const spaces = settings.tabSize - (value.index % settings.tabSize)

        editor().put(value, ' '.repeat(spaces))

        putCursor({ line: value.line, index: value.index + spaces })
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

      // Empty lines should not affect whether we comment.
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

    const lineOffset = value.highlight ? 0 : 1

    if (all) {
      editor().crop(start, crops)

      putCursor({ line: value.line + lineOffset, index: cursorIndex })

      if (value.highlight && highlightIndex) {
        putCursor({
          line: value.highlight.line, index: highlightIndex
        }, value.highlight)
      }
    } else {
      editor().prefix(start, end, '# ', true)

      // This should... maybe be correct cursor positioning
      const { leading: cursorLeading } = grabWhitespace(editor().lineAt(value.line))
      if (value.index >= cursorLeading.length) {
        putCursor({ line: value.line + lineOffset, index: value.index + 2 })
      } else {
        putCursor({ line: value.line + lineOffset, index: value.index })
      }

      if (value.highlight) {
        const { leading: highlightLeading } = grabWhitespace(editor().lineAt(value.highlight.line))

        if (value.highlight.index >= highlightLeading.length) {
          putCursor({
            line: value.highlight.line, index: value.highlight.index + 2
          }, value.highlight)
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
          value.highlight = alignCursor({ line: end, index: text.length })
        }

        break

      case '/':
        comment()
        break

      case 'z': {
        if (event.shiftKey) {
          return // do nothing, ignore "redo"
        }

        const frame = editor().undo()

        suggestions?.dismissSuggestions()

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
        moveLeft(hasAltKey(event), event.shiftKey)
        break

      case 'ArrowRight':
        moveRight(hasAltKey(event), event.shiftKey)
        break

      case 'ArrowDown':
        moveDown(event.shiftKey)
        break

      case 'ArrowUp':
        moveUp(event.shiftKey)
        break

      case 'Escape':
        if (suggestions && suggestions.hasSuggestions()) {
          suggestions.dismissSuggestions()
        }

        event.preventDefault()

        break

      case 'Tab':
        event.preventDefault()

        if (!event.shiftKey && suggestions && suggestions.flushSuggestions()) {
          const suggestion = suggestions.mergeSuggestion()

          if (suggestion) {
            return applyMergeSuggestion(suggestion)
          }
        } else {
          hitTab(event.shiftKey)
        }

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
            nextPosition = editor().deleteForwards(value, hasAltKey(event))
          } else {
            nextPosition = editor().backspace(value, hasAltKey(event), settings.tabSize)
          }

          putCursor(nextPosition)
        }

        promptSuggestions()

        break

      case 'Enter':
        if (event.shiftKey) {
          putCursor({
            line: value.line,
            index: editor().lineAt(value.line).length
          })

          suggestions?.dismissSuggestions()
        } else if (suggestions && suggestions.flushSuggestions()) {
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

          if (showSuggestionsAt) {
            showSuggestionsAt(cursor())
          }
        }

        break
    }
  }

  function cursorCoordinates(x: number, y: number): SelectionIndex {
    const count = editor().lineCount()

    if (count <= 0) {
      return { line: 0, index: 0 }
    }

    const lineIndex = Math.floor(y / lineHeight)
    const line = Math.min(Math.max(lineIndex, 0), count - 1)
    const text = editor().lineAt(line)

    const index = calculator.position(text, x)

    return { line, index }
  }

  function dropCursor(x: number, y: number, detail?: number, shift?: boolean) {
    const index = cursorCoordinates(x, y)

    if (detail === 2) {
      const line = editor().lineAt(index.line)

      function isSpace(text?: string) {
        if (text) {
          return /\s/.test(text)
        }

        return false
      }

      const backwardSpace = isSpace(line[index.index - 1])
      const forwardSpace = isSpace(line[index.index])

      const directional = backwardSpace !== forwardSpace

      const backward = directional && backwardSpace ? 0 : consumeDirection(line, index.index, -1, false)
      const forward = directional && forwardSpace ? 0 : consumeDirection(line, index.index, +1, false)

      const backwardIndex = { index: index.index - backward, line: index.line }
      const forwardIndex = { index: index.index + forward, line: index.line }

      const current = cursor()
      putCursor(forwardIndex, current)
      current.highlight = alignCursor(backwardIndex)
    } else if (detail === 3) {
      const line = editor().lineAt(index.line)

      const start = { index: 0, line: index.line }
      const end = { index: line.length, line: index.line }

      const current = cursor()
      putCursor(end, current)
      current.highlight = alignCursor(start)
    } else {
      const value = cursor()

      if (shift) {
        if (!value.highlight) {
          value.highlight = { line: value.line, index: value.index }
        }
      } else {
        value.highlight = null
      }

      editor().commit()
      suggestions?.dismissSuggestions()

      putCursor(index)
    }
  }

  function dragTo(x: number, y: number) {
    const value = cursor()

    makeSelection()
    putCursor(cursorCoordinates(x, y))

    if (value.highlight
      && value.highlight.line === value.line
      && value.highlight.index === value.index) {
      value.highlight = null
    }
  }

  function jump(index: SelectionIndex) {
    putCursor(index)

    cursor().highlight = null
  }

  function lineStart(line: number): number {
    return line * lineHeight
  }

  return {
    range,
    position,
    jump,
    lineStart,
    getSelection,
    dropSelection,
    dropCursor,
    cursorCoordinates,
    dragTo,
    pasteText,
    handleKey,
    applyMergeSuggestion
  }
}
