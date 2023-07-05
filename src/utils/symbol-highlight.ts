import { grabWhitespace, Token, TokenType } from './languages/language'
import { reactive, ref } from 'vue'
import { StorageResult } from './storage'
import { CursorPosition } from './cursor'
import { findToken } from './languages/suggestions'
import { SelectionIndex } from './editor'

export interface SymbolHighlightInterface {
  update(token: Token, line: number): void
  clear(): void
  updateCursor(cursor: SelectionIndex): void
  highlights(start: number, count: number): SymbolHighlight[][] | null
  symbolName(name: string): string
}

export interface SymbolHighlight {
  index: number
  offset: number
  size: number
}

export type SymbolHighlightResult = SymbolHighlightInterface

export type WidthQuery = (text: string) => number

interface BracketId {
  line: number
  id: number
}

export function useSymbolHighlight(
  storage: StorageResult,
  widthQuery: WidthQuery
): SymbolHighlightResult {
  let bracket = null as BracketId | null
  let name = null as string | null
  let cache = ref(null as (SymbolHighlight[] | null)[] | null)

  function matches(other: Token, line: number): boolean {
    return (bracket !== null
        && line === bracket.line
        && bracket.id === other.bracket)
      || (name !== null
        && symbolName(other.text) === name)
  }

  function matchingTokens(
    tokens: Token[],
    line: number
  ): SymbolHighlight[] {
    const result = [] as SymbolHighlight[]

    for (const [index, token] of tokens.entries()) {
      if (matches(token, line)) {
        const leadingTokenText = tokens.slice(0, index).map(x => x.text).join('')
        const { leading } = grabWhitespace(token.text)

        const leadingText = leadingTokenText + leading

        const leadingOffset = widthQuery(leadingText)

        result.push({
          index: leadingText.length,
          offset: leadingOffset,
          size: widthQuery(token.text.substring(leading.length))
        })
      }
    }

    return result
  }

  function symbolName(text: string): string {
    let trim = text.trim()

    if (trim.endsWith(':')) {
      trim = trim.substring(0, trim.length - 1).trim()
    }

    return trim
  }

  function clear() {
    bracket = null
    name = null

    cache.value = null
  }

  function update(token: Token, line: number) {
    if (token.bracket !== undefined) {
      bracket = { line, id: token.bracket }
    } else {
      bracket = null
    }

    if (token.type === TokenType.Symbol || token.type === TokenType.Label) {
      name = symbolName(token.text)
    } else {
      name = null
    }

    cache.value = Array(storage.storage.highlights.length).map(() => null)
  }

  function updateCursor(cursor: SelectionIndex) {
    const tokens = storage.storage.highlights[cursor.line]
    const token = findToken(tokens, cursor.index)

    if (token) {
      update(token, cursor.line)
    } else {
      clear()
    }
  }

  function highlights(start: number, count: number): SymbolHighlight[][] | null {
    const lines = cache.value
    if (!lines) {
      return null
    }

    const result = Array(count)

    for (let i = 0; i < count; i++) {
      const line = i + start

      if (!lines[line]) {
        lines[line] = matchingTokens(storage.storage.highlights[line], line)
      }

      result[i] = lines[line]
    }

    return result
  }

  return {
    update,
    clear,
    updateCursor,
    highlights,
    symbolName
  }
}
