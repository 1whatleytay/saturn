import { findTokenIndex } from './languages/suggestions'
import { grabWhitespace, Token } from './languages/language'
import { reactive } from 'vue'

export interface Highlights {
  line: number
  offset: number
  size: number
  message: string
}

export interface HighlightsInterface {
  setHighlight(line: number, tokens: Token[], index: number, message: string): void,
  dismissHighlight(): void
  shiftHighlight(line: number, deleted: number, replaced: number): void
}

export type HighlightsResult = HighlightsInterface & {
  state: {
    highlight: Highlights | null
  }
}

export function useHighlights(
  widthQuery: (text: string) => number
): HighlightsResult {
  const state = reactive({
    highlight: null as Highlights | null
  })

  function setHighlight(line: number, tokens: Token[], index: number, message: string) {
    const tokenIndex = findTokenIndex(tokens, index + 1)

    if (tokenIndex === null) {
      return
    }

    const token = tokens[tokenIndex]

    const { leading, trailing } = grabWhitespace(token.text)

    const offset = widthQuery(tokens.slice(0, tokenIndex).map(x => x.text).join('') + leading)
    const size = widthQuery(token.text.substring(leading.length, token.text.length - trailing.length))

    state.highlight = { line, message, offset, size }
  }

  function shiftHighlight(line: number, deleted: number, replaced: number) {
    if (state.highlight) {
      if (line <= state.highlight.line && state.highlight.line < line + deleted) {
        state.highlight = null
      } else if (replaced !== deleted && state.highlight.line >= line + deleted) {
        state.highlight.line += replaced - deleted
      }
    }
  }

  function dismissHighlight() {
    state.highlight = null
  }

  return {
    state,
    setHighlight,
    shiftHighlight,
    dismissHighlight
  }
}
