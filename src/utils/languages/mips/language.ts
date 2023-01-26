import { Language, Token, TokenType } from '../language'
import { Suggestion, SuggestionMatch } from '../suggestions'
import { lex } from './lexer'
import { directives, lineSuggestions, registers, toSuggestionMatches } from './suggestions'

export class MipsHighlighter implements Language {
  highlight(line: string): Token[] {
    return lex(line)
  }

  inspect(tokens: Token[]): Suggestion[] {
    return []
  }

  suggest(token: Token): SuggestionMatch[] {
    switch (token.type) {
      case TokenType.Instruction: {
        const results = lineSuggestions.search(token.text.trim())

        return toSuggestionMatches(results)
      }

      case TokenType.Register: {
        const results = registers.search(token.text.trim())

        return toSuggestionMatches(results)
      }

      case TokenType.Directive: {
        const results = directives.search(token.text.trim())

        return toSuggestionMatches(results)
      }

      default:
        return []
    }
  }
}
