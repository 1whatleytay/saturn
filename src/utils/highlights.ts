import { findTokenIndex } from './languages/suggestions'
import { grabWhitespace, Token } from './languages/language'
import { reactive, UnwrapRef } from 'vue'

type DefaultMessage = string

export interface Highlights<Message = DefaultMessage> {
  line: number
  offset: number
  size: number
  message: Message
}

export interface HighlightsInterface<Message = DefaultMessage> {
  setHighlight(line: number, tokens: Token[], index: number, message: UnwrapRef<Message>): void
  putHighlight(line: number, tokenIndex: number, tokens: Token[], message: UnwrapRef<Message>): void
  dismissHighlight(): void
  shiftHighlight(line: number, deleted: number, replaced: number): void
}

export interface HighlightsState<Message = DefaultMessage> {
  highlight: Highlights<UnwrapRef<Message>> | null
}

export type HighlightsResult<Message = DefaultMessage> = HighlightsInterface<Message> & {
  state: HighlightsState<Message>
}

export function useHighlights<Message = DefaultMessage>(
  widthQuery: (text: string) => number
): HighlightsResult<Message> {
  const state = reactive({
    highlight: null as Highlights<Message> | null
  })

  function putHighlight(line: number, tokenIndex: number, tokens: Token[], message: UnwrapRef<Message>) {
    const token = tokens[tokenIndex]

    const { leading, trailing } = grabWhitespace(token.text)

    const offset = widthQuery(tokens.slice(0, tokenIndex).map(x => x.text).join('') + leading)
    const size = widthQuery(token.text.substring(leading.length, token.text.length - trailing.length))

    state.highlight = { line, message, offset, size }
  }

  function setHighlight(line: number, tokens: Token[], index: number, message: UnwrapRef<Message>) {
    const tokenIndex = findTokenIndex(tokens, index + 1)

    if (tokenIndex === null) {
      return
    }

    putHighlight(line, tokenIndex, tokens, message)
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
    putHighlight,
    shiftHighlight,
    dismissHighlight
  }
}
