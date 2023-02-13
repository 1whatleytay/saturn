import { reactive, watch } from 'vue'
import { configureDisplay } from './mips'

const settingsVersion = 1

export interface BitmapSettings {
  width: number
  height: number
  address: number
}

export interface EditorSettings {
  tabSize: number
}

export interface Settings {
  version: number // 1
  editor: EditorSettings
  bitmap: BitmapSettings
}

function defaultSettings(): Settings {
  return {
    version: 1,
    editor: {
      tabSize: 4
    },
    bitmap: {
      width: 64,
      height: 64,
      address: 0x10008000
    }
  }
}

const storageKey = 'saturn:settings'
function fromStorage(): Settings {
  const value = localStorage.getItem(storageKey)

  if (value) {
    const object = JSON.parse(value) as Settings // Just cast and hope.

    if (object.version === settingsVersion) {
      return object
    }
  }

  return defaultSettings()
}

function toStorage(settings: Settings) {
  const value = JSON.stringify(settings)

  localStorage.setItem(storageKey, value)
}

export function useSettings(): Settings {
  const state = reactive(fromStorage())

  configureDisplay(state.bitmap).then(() => { })

  watch(() => state.bitmap, async bitmap => {
    // Update tauri backend.
    await configureDisplay(bitmap)
  }, { deep: true })

  let debounce = null as number | null

  watch(() => state, settings => {
    if (debounce) {
      window.clearTimeout(debounce)
    }

    debounce = window.setTimeout(() => toStorage(settings), 1000)
  }, { deep: true })

  return state
}
