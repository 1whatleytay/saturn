import { HighlightResult, Language, Token, TokenType } from '../language'
import { SuggestionMatch } from '../suggestions'
import { lex } from './lexer'
import { directives, instructions, mergeResults, registers, registersSet, toSuggestionMatches } from './suggestions'

export class MipsHighlighter implements Language {
  highlight(line: string): HighlightResult {
    return lex(line)
  }

  suggest(token: Token): SuggestionMatch[] {
    switch (token.type) {
      case TokenType.Instruction: {
        const trim = token.text.trim()
        const instructionResults = instructions.search(trim)
        const directiveResults = directives.search(trim)

        return toSuggestionMatches(mergeResults(instructionResults, directiveResults))
      }

      case TokenType.Register: {
        const text = token.text.trim()

        if (registersSet.has(text)) {
          return []
        }

        const results = registers.search(text)

        return toSuggestionMatches(results)
      }

      case TokenType.Directive: {
        const text = token.text.trim()

        const results = directives.search(text)

        return toSuggestionMatches(results)
      }

      default:
        return []
    }
  }
}
