import { reactive, watch } from 'vue'
import { EditorTab } from './tabs'
import { Token } from './languages/language'
import { HighlightsInterface } from './highlights'
import { backend } from '../state/backend'

export interface StorageState {
  highlights: Token[][]
  debounce: number | null
}

export type StorageResult = {
  storage: StorageState
}

export function useStorage(
  error: HighlightsInterface,
  tab: () => EditorTab | null,
): StorageResult {
  const storage = reactive({
    highlights: [] as Token[][],
    debounce: null as number | null,
  } as StorageState)

  async function checkSyntax() {
    const current = tab()

    const result = await backend.assembleText(
      current?.doc.toString() ?? '',
      current?.path ?? null,
    )

    if (result.status === 'Error' && result.marker) {
      error.setHighlight(
        result.marker.line,
        result.marker.offset,
        result.message,
      )
    } else {
      error.dismissHighlight()
    }
  }

  function dispatchCheckSyntax() {
    if (tab()?.profile?.kind === 'asm') {
      if (storage.debounce) {
        window.clearTimeout(storage.debounce)
      }

      storage.debounce = window.setTimeout(checkSyntax, 500)
    }
  }

  function highlightAll() {
    // Might need to look at tab file extension to pick languages
    storage.highlights = [] // needs highlighting here

    dispatchCheckSyntax()

    error.dismissHighlight()
  }

  watch(
    () => tab()?.doc,
    () => highlightAll(),
    { immediate: true },
  )

  return {
    storage,
  } as StorageResult
}
