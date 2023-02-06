import { reactive, watch } from 'vue'
import { configureDisplay } from './mips'

export interface BitmapSettings {
  width: number
  height: number
  address: number
}

export interface Settings {
  tabSize: number,
  bitmap: BitmapSettings
}

function defaultSettings(): Settings {
  return {
    tabSize: 4,
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
    return JSON.parse(value) // Just cast and hope.
  } else {
    return defaultSettings()
  }
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
