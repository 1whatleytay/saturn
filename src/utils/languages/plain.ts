import { Highlighter, style, Token, TokenType } from './highlighter'

export class PlainHighlighter implements Highlighter {
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
}
