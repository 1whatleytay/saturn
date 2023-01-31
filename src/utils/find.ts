import { reactive, watch } from 'vue'

export interface FindMatch {
  offset: number
  size: number
}

export interface FindState {
  show: boolean
  focus: boolean
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

  let size = null as number | null

  while (index >= 0) {
    lastOffset += widthQuery(line.substring(lastIndex, index))

    if (!size) {
      size = widthQuery(pin)
    }

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
    focus: false,
    text: '',
    matches: [],
    debounce: null as number | null
  } as FindState & { debounce: number | null })

  async function matchesFor(lines: string[]): Promise<FindMatch[][]> {
    return lines
      .map(value => findPin(value, state.text, widthQuery))
  }

  async function dirty(line: number, deleted: number, lines: string[]) {
    if (!state.show) {
      return
    }

    const results = await matchesFor(lines)

    state.matches.splice(line, deleted, ...results)
  }

  async function findAll() {
    if (state.show) {
      state.matches = await matchesFor(lines())
    }
  }

  // Intentionally avoiding using computed.
  watch(() => state.show, value => {
    if (value) {
      findAll().then(() => { /* nothing */ })
    } else {
      state.matches = []
    }
  })

  // Not deep, which is good!
  watch(() => lines(), findAll)
  watch(() => state.text, () => {
    if (state.debounce) {
      window.clearTimeout(state.debounce)
    }

    state.debounce = window.setTimeout(findAll, 200)
  })

  return {
    state,
    dirty
  }
}
