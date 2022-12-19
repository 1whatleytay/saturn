import { computed, reactive } from 'vue'

import { tab } from './editor-state'

import { regular } from '../utils/text-size'
import { consumeBackwards, consumeForwards } from '../utils/alt-consume'

export const lines = computed(() => tab()?.lines ?? ['Nothing yet.'])

export const cursor = reactive({
  line: 0,
  index: 0,
  offsetX: 0,
  offsetY: 0
})

export function putCursor(line: number, index: number) {
  let underflow = false
  let overflow = false

  let actualLine = line

  if (actualLine >= lines.value.length) {
    overflow = true
    actualLine = lines.value.length - 1
  } else if (actualLine < 0) {
    underflow = true
    actualLine = 0
  }

  const text = lines.value[actualLine]

  let actualIndex: number

  if (underflow) {
    actualIndex = 0
  } else if (overflow) {
    actualIndex = text.length
  } else {
    actualIndex = Math.max(index, 0)
  }

  const leading = text.substring(0, actualIndex)
  const size = regular.calculate(leading)

  cursor.line = actualLine
  cursor.index = actualIndex
  cursor.offsetX = size.width
  cursor.offsetY = size.height * actualLine
}

function insert(text: string) {
  const all = lines.value

  const line = all[cursor.line]
  const leading = line.substring(0, cursor.index)
  const trailing = line.substring(cursor.index)

  // Mutate
  all[cursor.line] = leading + text + trailing
  putCursor(cursor.line, cursor.index + text.length)
}

function newline() {
  const all = lines.value

  const line = all[cursor.line]
  const leading = line.substring(0, cursor.index)
  const trailing = line.substring(cursor.index)

  // Mutate
  all[cursor.line] = leading
  all.splice(cursor.line + 1, 0, trailing)

  putCursor(cursor.line + 1, 0)
}

function backspace(alt: boolean = false) {
  const all = lines.value

  const line = all[cursor.line]

  if (cursor.index > 0) {
    const consumption = alt ? consumeBackwards(line, cursor.index) : 1

    const leading = line.substring(0, cursor.index - consumption)
    const trailing = line.substring(cursor.index)

    // Mutate
    all[cursor.line] = leading + trailing

    putCursor(cursor.line, cursor.index - consumption)
  } else if(cursor.line > 0) {
    const leading = all[cursor.line - 1]
    const trailing = all[cursor.line]

    all[cursor.line - 1] = leading + trailing

    all.splice(cursor.line, 1)

    putCursor(cursor.line - 1, leading.length)
  }
}

function moveLeft(alt: boolean = false) {
  const consume = alt ? consumeBackwards(lines.value[cursor.line], cursor.index) : 1

  let line = cursor.line
  let move = cursor.index - consume

  if (move < 0) {
    line -= 1
    move = lines.value[line].length
  }

  putCursor(line, move)
}

function moveRight(alt: boolean = false) {
  const consume = alt ? consumeForwards(lines.value[cursor.line], cursor.index) : 1

  let line = cursor.line
  let move = cursor.index + consume

  if (lines.value[cursor.line].length < move) {
    line += 1
    move = 0
  }

  putCursor(line, move)
}

function moveDown() {
  putCursor(cursor.line + 1, cursor.index)
}

function moveUp() {
  putCursor(cursor.line - 1, cursor.index)
}

export function handleKey(event: KeyboardEvent) {
  switch (event.key) {
    case 'ArrowLeft':
      moveLeft(event.altKey)
      break

    case 'ArrowRight':
      moveRight(event.altKey)
      break

    case 'ArrowDown':
      moveDown()
      break

    case 'ArrowUp':
      moveUp()
      break

    case 'Backspace':
      backspace(event.altKey)
      break

    case 'Enter':
      newline()
      break

    default:
      if (event.metaKey || event.ctrlKey) {
        /* handle meta */
      } else if (event.key.length === 1) {
        insert(event.key)
      }

      break
  }
}

const defaultCursorPush = 4
export function dropCursor(x: number, y: number) {
  const { height } = regular.calculate('')

  if (lines.value.length <= 0) {
    return
  }

  const lineIndex = Math.floor(y / height)
  const line = Math.min(Math.max(lineIndex, 0), lines.value.length - 1)
  const text = lines.value[line]

  const index = regular.position(text, x, defaultCursorPush)

  putCursor(line, index)
}
