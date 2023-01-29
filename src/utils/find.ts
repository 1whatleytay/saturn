import { reactive } from 'vue'

export interface FindState {
  show: boolean
  text: string
}

export interface FindResult {
  state: FindState
}

export function useFind(): FindResult {
  const state = reactive({
    show: false,
    text: ''
  })

  return {
    state
  }
}
