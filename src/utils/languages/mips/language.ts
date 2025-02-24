import { HighlightResult, Language } from '../language'
import { lex } from './lexer'

export class MipsHighlighter implements Language {
  highlight(line: string): HighlightResult {
    return lex(line)
  }
}
