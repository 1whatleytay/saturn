import { HighlightResult, Language, style, TokenType } from './language'

export class PlainHighlighter implements Language {
  highlight(line: string): HighlightResult {
    return {
      tokens: [
        {
          start: 0,
          text: line,
          type: TokenType.Nothing,
          color: style.nothing,
        },
      ],
      suggestions: [],
    }
  }
}
