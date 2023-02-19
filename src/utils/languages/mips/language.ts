import { HighlightResult, Language, Token, TokenType } from '../language'
import { SuggestionMatch, SuggestionsStorage, SuggestionType } from '../suggestions'
import { lex } from './lexer'
import {
  directives,
  fuseOptions,
  instructions,
  mergeResults,
  registers,
  registersSet,
  toSuggestionMatches
} from './suggestions'
import Fuse from 'fuse.js'

export class MipsHighlighter implements Language {
  highlight(line: string): HighlightResult {
    return lex(line)
  }

  suggest(token: Token, storage?: SuggestionsStorage): SuggestionMatch[] {
    switch (token.type) {
      case TokenType.Instruction: {
        const trim = token.text.trim()
        const instructionResults = instructions.search(trim)
        const directiveResults = directives.search(trim)

        const fuse = storage?.cache('macros', values => {
          const filter = Array.from(values).filter(s => s.type === SuggestionType.Function)

          return new Fuse(filter, fuseOptions)
        })

        const macroResults = fuse?.search(trim) ?? []

        return toSuggestionMatches(mergeResults(instructionResults, directiveResults, macroResults))
      }

      case TokenType.Symbol: {
        if (storage) {
          const trim = token.text.trim()
          const fuse = storage.cache('symbols', values => {
            const filter = Array.from(values).filter(
              s => s.type === SuggestionType.Label || s.type === SuggestionType.Variable)

            return new Fuse(filter, fuseOptions)
          })

          const results = fuse.search(trim)

          return toSuggestionMatches(results)
        }

        return []
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
