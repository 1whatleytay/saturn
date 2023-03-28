import { reactive } from 'vue'
import { EditorTab } from './tabs'

export interface SaveModalState {
  tab: EditorTab | null
}

export interface SaveModalResult {
  state: SaveModalState

  present(tab: EditorTab): void

  selectSave(): void
  selectDiscard(): void
  selectDismiss(): void
}

export function useSaveModal(
  save: (tab: EditorTab) => Promise<boolean>,
  close: (tab: EditorTab) => void
): SaveModalResult {
  const state = reactive({
    tab: null,
  } as SaveModalState)

  function present(tab: EditorTab) {
    if (!state.tab) {
      state.tab = tab
    }
  }

  async function selectSave() {
    if (state.tab) {
      if (await save(state.tab)) {
        close(state.tab)
      }

      state.tab = null
    }
  }

  function selectDiscard() {
    if (state.tab) {
      close(state.tab)

      state.tab = null
    }
  }

  function selectDismiss() {
    state.tab = null
  }

  return {
    state,
    present,
    selectSave,
    selectDiscard,
    selectDismiss,
  }
}
