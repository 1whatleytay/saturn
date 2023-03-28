import { HighlightsResult } from './highlights'
import {
  findTokenIndex,
  MarkedSuggestion,
  SuggestionType,
} from './languages/suggestions'
import { StorageResult } from './storage'
import { SelectionIndex } from './editor'
import { Token } from './languages/language'

export interface GotoMessage {
  label: string
  line: number
  index: number
  type?: SuggestionType
}

export interface GotoInterface {
  dismiss(): void
  hover(line: number, index: number): void
  jump(): SelectionIndex | null
}

interface SearchStorageResult {
  suggestion: MarkedSuggestion
  line: number
}

interface GotoCache {
  line: number
  index: number // tokenIndex
  token: Token
  match: SearchStorageResult | null
}

export function useGoto(
  highlights: HighlightsResult<GotoMessage>,
  storage: StorageResult
): GotoInterface {
  function searchStorage(
    text: string,
    body: MarkedSuggestion[][]
  ): SearchStorageResult | null {
    for (const [index, line] of body.entries()) {
      const item = line.find((x) => x.replace == text)

      if (item) {
        return { line: index, suggestion: item }
      }
    }

    return null
  }

  let cache = null as GotoCache | null

  function dismiss() {
    cache = null
    highlights.dismissHighlight()
  }

  function hover(line: number, index: number) {
    highlights.dismissHighlight()

    const tokens = storage.storage.highlights[line]
    const tokenIndex = findTokenIndex(tokens, index)

    if (tokenIndex === null) {
      return
    }

    let match: SearchStorageResult | null

    if (cache && cache.line === line && cache.index === tokenIndex) {
      match = cache.match
    } else {
      const token = tokens[tokenIndex]
      const text = token.text.trim()

      const suggestions = storage.suggestionsStorage()
      // I don't care about the performance of this feature too much
      match = searchStorage(text, suggestions.body)

      cache = {
        line: line,
        index: tokenIndex,
        token,
        match,
      }
    }

    if (!match) {
      return
    }

    const message: GotoMessage = {
      label: match.suggestion.replace,
      line: match.line,
      index: match.suggestion.index,
      type: match.suggestion.type,
    }

    highlights.putHighlight(line, tokenIndex, tokens, message)
  }

  function jump(): SelectionIndex | null {
    highlights.dismissHighlight()

    // Relying on cache is probably not great, but no point in looking up things again.
    if (!cache || !cache.match) {
      return null
    }

    return {
      line: cache.match.line,
      index: cache.match.suggestion.index,
    }
  }

  return {
    dismiss,
    hover,
    jump,
  }
}
