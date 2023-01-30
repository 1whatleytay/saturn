import { reactive, watch } from 'vue'

export interface FindMatch {
  offset: number
  size: number
}

export interface FindState {
  show: boolean
  text: string
  matches: FindMatch[][]
}

export interface FindInterface {
  dirty(line: number, deleted: number, lines: string[]): void
}

export type FindResult = FindInterface & {
  state: FindState
}

export type WidthQuery = (text: string) => number

function findPin(line: string, pin: string, widthQuery: WidthQuery): FindMatch[] {
  if (!pin.length) {
    return []
  }

  const result = [] as FindMatch[]

  let index = line.indexOf(pin)

  let lastIndex = 0
  let lastOffset = 0

  const size = widthQuery(pin)

  while (index >= 0) {
    lastOffset += widthQuery(line.substring(lastIndex, index))

    result.push({
      offset: lastOffset,
      size
    })

    lastOffset += size
    lastIndex = index + pin.length

    index = line.indexOf(pin, lastIndex)
  }

  return result
}

export function useFind(lines: () => string[], widthQuery: WidthQuery): FindResult {
  const state = reactive({
    show: false,
    text: '',
    matches: []
  } as FindState)

  function matchesFor(line: number, lines: string[]): FindMatch[][] {
    return lines
      .map(value => findPin(value, state.text, widthQuery))
  }

  function dirty(line: number, deleted: number, lines: string[]) {
    if (!state.show) {
      return
    }

    state.matches.splice(line, deleted, ...matchesFor(line, lines))
  }

  function findAll() {
    if (state.show) {
      state.matches = matchesFor(0, lines())
    }
  }

  // Intentionally avoiding using computed.
  watch(() => state.show, value => {
    if (value) {
      findAll()
    } else {
      state.matches = []
    }
  })

  // Not deep, which is good!
  watch(() => lines(), findAll)
  watch(() => state.text, findAll)

  return {
    state,
    dirty
  }
}
