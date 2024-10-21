import {
  MarkedSuggestion,
  SuggestionMatch,
  SuggestionsStorage,
} from './suggestions'

export enum TokenType {
  Comment,
  Hard,
  BracketOpen,
  BracketClose,
  Label,
  Directive,
  Parameter,
  Instruction,
  Register,
  Numeric,
  Symbol,
  Text,
  Nothing,
}

export interface Token {
  start: number
  text: string
  type: TokenType
  color: string
  bracket?: number
}

export const style = {
  known: 'font-bold',
  comment: 'dark:text-neutral-400 text-neutral-500',
  hard: 'dark:text-yellow-400 text-yellow-700',
  label: 'dark:text-amber-400 text-amber-700',
  directive: 'dark:text-red-400 text-red-700',
  parameter: 'dark:text-orange-300 text-orange-800',
  instruction: 'dark:text-sky-400 text-sky-700',
  register: 'dark:text-orange-300 text-orange-800',
  numeric: 'dark:text-teal-300 text-teal-800',
  symbol: 'dark:text-white text-black',
  text: 'dark:text-lime-300 text-lime-800',
  nothing: 'dark:text-white text-black',
}

function mapStyle(type: TokenType): string {
  switch (type) {
    case TokenType.Comment:
      return style.comment
    case TokenType.Hard:
      return style.hard
    case TokenType.Label:
      return style.label
    case TokenType.Directive:
      return style.directive
    case TokenType.Parameter:
      return style.parameter
    case TokenType.Instruction:
      return style.instruction
    case TokenType.Register:
      return style.register
    case TokenType.Numeric:
      return style.numeric
    case TokenType.Symbol:
      return style.symbol
    case TokenType.Text:
      return style.text
    case TokenType.BracketOpen:
    case TokenType.BracketClose:
    case TokenType.Nothing:
    default:
      return style.nothing
  }
}

export function getStyle(type: TokenType, known: boolean = false): string {
  const value = mapStyle(type)

  if (known) {
    return `${value} ${style.known}`
  } else {
    return value
  }
}

export interface HighlightResult {
  tokens: Token[]
  suggestions: MarkedSuggestion[]
}

export interface Language {
  highlight(line: string): HighlightResult

  // for line while user is typing
  suggest(token: Token, storage?: SuggestionsStorage): SuggestionMatch[]
}

// For Tokens
export function grabWhitespace(text: string): {
  leading: string
  trailing: string
} {
  const leadingMatches = text.match(/^\s*/)
  const trailingMatches = text.match(/\s*$/)

  return {
    leading: leadingMatches && leadingMatches.length ? leadingMatches[0] : '',
    trailing:
      trailingMatches && trailingMatches.length ? trailingMatches[0] : '',
  }
}
