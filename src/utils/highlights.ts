import { reactive, UnwrapRef } from 'vue'

type DefaultMessage = string

export interface Highlights<Message = DefaultMessage> {
  line: number
  offset: number
  message: Message
}

export interface HighlightsInterface<Message = DefaultMessage> {
  setHighlight(
    line: number,
    tokenIndex: number,
    message: UnwrapRef<Message>
  ): void
  dismissHighlight(): void
}

export interface HighlightsState<Message = DefaultMessage> {
  highlight: Highlights<UnwrapRef<Message>> | null
}

export type HighlightsResult<Message = DefaultMessage> =
  HighlightsInterface<Message> & {
    state: HighlightsState<Message>
  }

export function useHighlights<Message = DefaultMessage>(): HighlightsResult<Message> {
  const state = reactive({
    highlight: null as Highlights<Message> | null,
  })
  function setHighlight(
    line: number,
    index: number,
    message: UnwrapRef<Message>
  ) {
    state.highlight = { line, message, offset: index }
  }

  function dismissHighlight() {
    state.highlight = null
  }

  return {
    state,
    setHighlight,
    dismissHighlight,
  }
}
