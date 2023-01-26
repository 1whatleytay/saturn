import { Language, Token, TokenType } from '../language'
import { Suggestion, SuggestionMatch } from '../suggestions'
import { lex } from './lexer'
import { instructions, registers } from './suggestions'
import Fuse from 'fuse.js'
import FuseResult = Fuse.FuseResult

function toSuggestionMatches(results: FuseResult<Suggestion>[]): SuggestionMatch[] {
  return results.map(x => {
    let range = undefined

    if (x.matches && x.matches.length > 0) {
      const match = x.matches[0]

      const first = match.indices[0]

      range = {
        start: first[0],
        end: first[1]
      }
    }

    return {
      ...x.item,
      range
    } as SuggestionMatch
  })
}

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
        const results = instructions.search(token.text.trim())

        return toSuggestionMatches(results)
      }

      case TokenType.Register: {
        const results = registers.search(token.text.trim())

        return toSuggestionMatches(results)
      }

      default:
        return []
    }
  }
}
