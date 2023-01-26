import { Token, TokenType } from './language'

export interface MatchRange {
  start: number
  end: number
}

export enum SuggestionType {
  Instruction
}

export interface Suggestion {
  replace: string
  name: string
  type?: SuggestionType
}

export type SuggestionMatch = Suggestion & { range?: MatchRange }

export function findToken(tokens: Token[], index: number): Token | null {
  let start = 0
  let end = tokens.length

  while (start < end) {
    const middle = Math.floor((end + start) / 2)
    const current = tokens[middle]

    // Cursor can be on token if it's at the end.
    if (current.start < index && index <= current.start + current.text.length) {
      return current
    }

    if (index <= current.start) {
      end = middle
    } else {
      start = middle + 1
    }
  }

  return null
}
