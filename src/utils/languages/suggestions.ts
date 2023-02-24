import { Token } from './language'

export interface MatchRange {
  start: number
  end: number
}

export enum SuggestionType {
  Instruction,
  Register,
  Directive,
  Label,
  Function, // Macro
  Variable
}

export interface Suggestion {
  replace: string
  name?: string
  type?: SuggestionType
}

export type MarkedSuggestion = Suggestion & { index: number }
export type SuggestionMatch = Suggestion & { range?: MatchRange }

export function findTokenIndex(tokens: Token[], index: number): number | null {
  let start = 0
  let end = tokens.length

  while (start < end) {
    const middle = Math.floor((end + start) / 2)
    const current = tokens[middle]

    // Cursor can be on token if it's at the end.
    if (current.start < index && index <= current.start + current.text.length) {
      return middle
    }

    if (index <= current.start) {
      end = middle
    } else {
      start = middle + 1
    }
  }

  return null
}

export function findToken(tokens: Token[], index: number): Token | null {
  const point = findTokenIndex(tokens, index)

  if (point === null) {
    return null
  }

  return tokens[point]
}

type IdentifiedSuggestion = MarkedSuggestion & { id: number }

export class SuggestionsStorage {
  id: number = 0 // incremented
  map = new Map<number, IdentifiedSuggestion>()
  body: IdentifiedSuggestion[][] = []

  cacheMap = new Map<string, any>()

  cache<T>(type: string, build: (values: IterableIterator<Suggestion>) => T): T {
    const result = this.cacheMap.get(type)

    if (result !== undefined) {
      return result
    }

    const value = build(this.map.values())
    this.cacheMap.set(type, value)

    return value
  }

  update(line: number, deleted: number, insert: MarkedSuggestion[][]) {
    const input = insert.map(l => l.map(s => ({ id: this.id++, ...s })))

    const drop = this.body.splice(line, deleted, ...input)

    // l -> line, s -> suggestion
    let mutated = false

    drop.forEach(l => l.forEach(s => {
      mutated = true
      this.map.delete(s.id)
    }))

    input.forEach(l => l.forEach(s => {
      mutated = true
      this.map.set(s.id, s)
    }))

    if (mutated) {
      this.cacheMap = new Map()
    }
  }
}
