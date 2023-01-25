import { Language, Token } from './languages/language'
import { findToken, Suggestion } from './languages/suggestions'
import { reactive } from 'vue'

export interface MergeSuggestion {
  start: number
  remove: number
  insert: string
}

export interface SuggestionsState {
  index: number
  token: Token | null
  results: Suggestion[]
}

export interface SuggestionsResult {
  suggestions: SuggestionsState
  moveIndex: (direction: number) => void
  showSuggestions: (tokens: Token[], index: number) => void
  mergeSuggestion: () => MergeSuggestion | null
  dismissSuggestions: () => void
  hasSuggestions: () => boolean // convenience
  flushSuggestions: () => boolean // flushes and returns hasSuggestions
}

export function useSuggestions(language: Language): SuggestionsResult {
  const suggestions = reactive({
    index: 0,
    token: null as Token | null,
    results: [] as Suggestion[],
    debounce: null as {
      interval: number,
      token: Token,
      end: number
    } | null
  })

  function makeSuggestions(token: Token) {
    console.log('making suggestions')

    suggestions.index = 0
    suggestions.token = token
    suggestions.results = language.suggest(token)

    suggestions.debounce = null
  }

  function showSuggestions(tokens: Token[], index: number) {
    const token = findToken(tokens, index)

    if (!token || !token.text.trim().length) {
      suggestions.results = []

      return
    }

    const debounceTimeout = 100
    const forceTimeout = 200

    const now = Date.now()

    if (suggestions.debounce) {
      window.clearInterval(suggestions.debounce.interval)

      if (suggestions.debounce.end <= now) {
        console.log('force')

        return makeSuggestions(token)
      }

      console.log('cancel')
    }

    suggestions.debounce = {
      interval: window.setTimeout(() => makeSuggestions(token), debounceTimeout),
      token,
      end: now + forceTimeout
    }
  }

  function moveCursor(direction: number) {
    const destination = suggestions.index + direction

    if (destination >= suggestions.results.length) {
      suggestions.index = 0
    } else if (destination < 0) {
      suggestions.index = suggestions.results.length - 1
    } else {
      suggestions.index = destination
    }
  }

  function grabWhitespace(text: string): { leading: string, trailing: string } {
    const leadingMatches = text.match(/^\s*/)
    const trailingMatches = text.match(/\s*$/)

    return {
      leading: leadingMatches && leadingMatches.length ? leadingMatches[0] : '',
      trailing: trailingMatches && trailingMatches.length ? trailingMatches[0] : ''
    }
  }

  function dismissSuggestions() {
    suggestions.token = null
    suggestions.index = 0
    suggestions.results = []
  }

  function mergeSuggestion(): MergeSuggestion | null {
    const token = suggestions.token
    const index = suggestions.index

    if (!token || index > suggestions.results.length) {
      return null
    }

    const suggestion = suggestions.results[index]

    const { leading, trailing } = grabWhitespace(token.text)

    return {
      start: token.start,
      remove: token.text.length,
      insert: leading + suggestion.replace + trailing
    }
  }

  function hasSuggestions(): boolean {
    return !!suggestions.results.length
  }

  function flushSuggestions(): boolean {
    if (suggestions.debounce) {
      window.clearInterval(suggestions.debounce.interval)

      makeSuggestions(suggestions.debounce.token)
    }

    return hasSuggestions()
  }

  return {
    suggestions,
    moveIndex: moveCursor,
    showSuggestions,
    dismissSuggestions,
    mergeSuggestion,
    hasSuggestions,
    flushSuggestions
  }
}