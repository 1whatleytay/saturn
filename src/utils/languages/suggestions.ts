import { Token } from './language'

export interface MatchRange {
  start: number
  end: number
}

export enum SuggestionType {
  Instruction,
  Register,
  Directive,
  Label
}

export interface Suggestion {
  replace: string
  name?: string
  type?: SuggestionType
}

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
