import { reactive, watch } from 'vue'
import { BitmapConfig } from './mips/mips'
import { backend } from '../state/console-data'

const settingsVersion = 6

export interface ExportRegionsOptions {
  kind: 'plain' | 'hex_v3'
  continuous: boolean
  encoding: 'byte' | 'big32' | 'little32'
}

export interface BitmapSettings {
  displayWidth: number
  displayHeight: number
  unitWidth: number
  unitHeight: number
  address: number
}

export interface EditorSettings {
  tabSize: number
  fontSize: number
  consoleFontSize: number
  enterAutocomplete: boolean
}

export enum RegisterFormat {
  Hexadecimal,
  Decimal,
  Signed,
}

export interface RegisterSettings {
  format: RegisterFormat
}

export interface ExecutionSettings {
  timeTravel: boolean
}

export enum AddressingMode {
  Byte,
  Half,
  Word,
}

export interface MemorySettings {
  address: string,
  mode: AddressingMode
}

export interface Settings {
  version: number
  editor: EditorSettings
  bitmap: BitmapSettings
  registers: RegisterSettings
  execution: ExecutionSettings
  memory: MemorySettings
  export: ExportRegionsOptions
}

function defaultSettings(): Settings {
  return {
    version: settingsVersion,
    editor: {
      tabSize: 4,
      fontSize: 22,
      consoleFontSize: 16,
      enterAutocomplete: true
    },
    bitmap: {
      displayWidth: 64,
      displayHeight: 64,
      unitWidth: 1,
      unitHeight: 1,
      address: 0x10008000,
    },
    registers: {
      format: RegisterFormat.Hexadecimal,
    },
    execution: {
      timeTravel: true
    },
    memory: {
      address: '0x10010000',
      mode: AddressingMode.Word,
    },
    export: {
      continuous: false,
      kind: 'hex_v3',
      encoding: 'little32'
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
    address: bitmap.address,
  }
}

export function useSettings(): Settings {
  const state = reactive(fromStorage())

  backend.configureDisplay(displayConfig(state.bitmap)).then(() => {})

  watch(
    () => state.bitmap,
    async (bitmap) => {
      // Update tauri backend.
      await backend.configureDisplay(displayConfig(bitmap))
    },
    { deep: true }
  )

  let debounce = null as number | null

  watch(
    () => state,
    (settings) => {
      if (debounce) {
        window.clearTimeout(debounce)
      }

      debounce = window.setTimeout(() => toStorage(settings), 1000)
    },
    { deep: true }
  )

  return state
}
