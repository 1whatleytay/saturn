import { Language, Token, TokenType } from '../language'
import { Suggestion } from '../suggestions'
import { lex } from './lexer'
import { instructions, suggestionFuse } from './suggestions'
import Fuse from 'fuse.js'

export class MipsHighlighter implements Language {
  highlight(line: string): Token[] {
    return lex(line)
  }

  inspect(tokens: Token[]): Suggestion[] {
    return []
  }

  suggest(token: Token): Suggestion[] {
    switch (token.type) {
      case TokenType.Instruction:
        const results = suggestionFuse.search(token.text.trim())

        return results.map(x => x.item)

      default:
        return []
    }
  }
}
