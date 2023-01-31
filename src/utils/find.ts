import { reactive, watch } from 'vue'

export interface FindMatch {
  index: number
  offset: number
  size: number
}

export interface FindState {
  show: boolean
  focus: boolean
  text: string
  matches: FindMatch[][]
  lastMatch: FindMatch | null
  paused: boolean
}

export interface FindNextItem {
  line: number
  match: FindMatch
}

export interface FindInterface {
  dirty(line: number, deleted: number, lines: string[]): void
  nextItem(line: number, index: number): FindNextItem | null
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

  let size = null as number | null

  while (index >= 0) {
    lastOffset += widthQuery(line.substring(lastIndex, index))

    if (!size) {
      size = widthQuery(pin)
    }

    result.push({
      index,
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
    focus: false,
    text: '',
    matches: [],
    lastMatch: null,
    paused: false, // true when item is last in list
    debounce: null
  } as FindState & { debounce: number | null })

  function matchesFor(lines: string[]): FindMatch[][] {
    return lines
      .map(value => findPin(value, state.text, widthQuery))
  }

  function dirty(line: number, deleted: number, lines: string[]) {
    if (!state.show) {
      return
    }

    const results = matchesFor(lines)

    state.matches.splice(line, deleted, ...results)
  }

  function findAll() {
    if (state.show) {
      state.matches = matchesFor(lines())
    }
  }

  function findNextIndex(matches: FindMatch[], index: number, exclude: FindMatch | null): number | null {
    let start = 0
    let end = matches.length

    while (start < end) {
      const middle = Math.floor((start + end) / 2)
      const item = matches[middle]

      const next = middle + 1 < matches.length ? matches[middle + 1] : null

      if (index < item.index) {
        end = middle
      } else if (index >= item.index) {
        if (!next || index < next.index) {
          if (item === exclude) {
            return next ? middle + 1 : null
          } else {
            return middle
          }
        }

        // next && the cursor is after next
        start = middle + 1
      }
    }

    return null
  }

  function nextItem(line: number, index: number): FindNextItem | null {
    const current = state.matches[line]
    const next = findNextIndex(current, index, state.lastMatch)

    if (next !== null) {
      const match = current[next]
      state.lastMatch = match

      return { line, match } // weird approach
    }

    function searchFrom(start: number): FindNextItem | null {
      for (let a = start; a < state.matches.length; a++) {
        const followup = state.matches[a]

        if (followup.length) {
          const match = followup[0]
          state.lastMatch = match

          return { line: a, match }
        }
      }

      return null
    }

    const result = searchFrom(line + 1)

    if (result) {
      state.paused = false
      return result
    }

    if (!state.paused) {
      state.paused = true
      return null // pause for a second at the last search
    }

    return searchFrom(0)
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
  watch(() => state.text, () => {
    state.lastMatch = null
    state.paused = true

    if (state.debounce) {
      window.clearTimeout(state.debounce)
    }

    state.debounce = window.setTimeout(findAll, 200)
  })

  return {
    state,
    dirty,
    nextItem
  }
}
