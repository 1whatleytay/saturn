import { reactive, watch } from 'vue'
import { BitmapConfig, configureDisplay } from './mips'

const settingsVersion = 3

export interface BitmapSettings {
  displayWidth: number
  displayHeight: number
  unitWidth: number
  unitHeight: number
  address: number
}

export interface EditorSettings {
  tabSize: number
}

export enum RegisterFormat {
  Hexadecimal,
  Decimal
}

export interface RegisterSettings {
  format: RegisterFormat
}

export interface Settings {
  version: number // 1
  editor: EditorSettings
  bitmap: BitmapSettings
  registers: RegisterSettings
}

function defaultSettings(): Settings {
  return {
    version: settingsVersion,
    editor: {
      tabSize: 4
    },
    bitmap: {
      displayWidth: 64,
      displayHeight: 64,
      unitWidth: 1,
      unitHeight: 1,
      address: 0x10008000
    },
    registers: {
      format: RegisterFormat.Hexadecimal
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

export function displayConfig(bitmap: BitmapSettings): BitmapConfig {
  return {
    width: Math.ceil(bitmap.displayWidth / bitmap.unitWidth),
    height: Math.ceil(bitmap.displayHeight / bitmap.unitHeight),
    address: bitmap.address
  }
}

export function useSettings(): Settings {
  const state = reactive(fromStorage())

  configureDisplay(displayConfig(state.bitmap)).then(() => { })

  watch(() => state.bitmap, async bitmap => {
    // Update tauri backend.
    await configureDisplay(displayConfig(bitmap))
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
