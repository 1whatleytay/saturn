import { findTokenIndex } from './languages/suggestions'
import { Token } from './languages/language'
import { reactive } from 'vue'

export interface ErrorHighlight {
  line: number
  offset: number
  size: number
  message: string
}

export interface ErrorHighlightResult {
  state: {
    highlight: ErrorHighlight | null
  },

  setHighlight: (line: number, index: number, message: string) => void
}

export function useErrorHighlight(
  widthQuery: (text: string) => number,
  tokensQuery: (line: number) => Token[] | null
): ErrorHighlightResult {
  const state = reactive({
    highlight: null as ErrorHighlight | null
  })

  function setHighlight(line: number, index: number, message: string) {
    const tokens = tokensQuery(line)

    if (!tokens) {
      return
    }

    const tokenIndex = findTokenIndex(tokens, index + 1)

    console.log({ tokenIndex, line, index })

    if (tokenIndex === null) {
      return
    }

    const token = tokens[tokenIndex]

    const offset = widthQuery(tokens.slice(0, tokenIndex).map(x => x.text).join(''))
    const size = widthQuery(token.text)

    state.highlight = { line, message, offset, size }
  }

  return {
    state,
    setHighlight
  }
}

