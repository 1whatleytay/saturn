import { Highlighter, style, Token } from './highlighter'

export class PlainHighlighter implements Highlighter {
  highlight(line: string): Token[] {
    return [{ text: line, color: style.nothing }]
  }
}
