import { Language, style, Token, TokenType } from './language'
import { Suggestion } from './suggestions'

export class PlainHighlighter implements Language {
  highlight(line: string): Token[] {
    return [
      {
        start: 0,
        text: line,
        type: TokenType.Nothing,
        color: style.nothing
      }
    ]
  }

  inspect(tokens: Token[]): Suggestion[] {
    return []
  }

  suggest(token: Token): Suggestion[] {
    return []
  }
}
