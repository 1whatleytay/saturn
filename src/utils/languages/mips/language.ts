import { Language, Token, TokenType } from '../language'
import { Suggestion, SuggestionMatch } from '../suggestions'
import { lex } from './lexer'
import { suggestionFuse } from './suggestions'

export class MipsHighlighter implements Language {
  highlight(line: string): Token[] {
    return lex(line)
  }

  inspect(tokens: Token[]): Suggestion[] {
    return []
  }

  suggest(token: Token): SuggestionMatch[] {
    switch (token.type) {
      case TokenType.Instruction:
        const results = suggestionFuse.search(token.text.trim())

        return results.map(x => {
          let range = undefined

          if (x.matches && x.matches.length === 2) {
            range = {
              start: x.matches[0],
              end: x.matches[1]
            }
          }

          return {
            ...x.item,
            range
          } as SuggestionMatch
        })

      default:
        return []
    }
  }
}
