import { reactive } from 'vue'

export interface BitmapSettings {
  width: number
  height: number
  address: number
}

export interface Settings {
  tabSize: number,
  bitmap: BitmapSettings
}

export function useSettings(): Settings {
  return reactive({
    tabSize: 4,
    bitmap: {
      width: 64,
      height: 64,
      address: 0x10008000
    }
  })
}
