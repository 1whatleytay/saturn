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
  comment: 'text-neutral-400',
  hard: 'text-yellow-400',
  label: 'text-amber-400',
  directive: 'text-red-400',
  parameter: 'text-orange-300',
  instruction: 'text-sky-400',
  register: 'text-orange-300',
  numeric: 'text-teal-300',
  symbol: 'text-white',
  text: 'text-lime-300',
  nothing: 'text-white',
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
