import { reactive } from 'vue'

export interface Settings {
  tabSize: number
}

export function useSettings(): Settings {
  return reactive({
    tabSize: 4
  })
}
