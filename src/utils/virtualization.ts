import { computed, ComputedRef, reactive } from 'vue'

export type UpdateVirtualization = (top: number, height: number) => void

export interface Virtualization {
  renderStart: ComputedRef<number>
  renderCount: ComputedRef<number>
  topPadding: ComputedRef<number>
  bottomPadding: ComputedRef<number>,
  getIndex: (index: number) => number
  update: UpdateVirtualization
}

const defaultLinePadding = 16
const defaultDangerPadding = 8

export function useVirtualize(
  lineHeight: number,
  count: () => number,
  linePadding: number = defaultLinePadding,
  dangerPadding: number = defaultDangerPadding
): Virtualization {
  const state = reactive({
    startIndex: 0,
    endIndex: 0
  })

  function inBounds(line: number) {
    return Math.max(Math.min(line, count()), 0)
  }

  function getIndex(i: number) {
    return i - 1 + renderStart.value
  }

  const renderStart = computed(() => inBounds(state.startIndex))
  const renderCount = computed(() => inBounds(state.endIndex) - renderStart.value)
  const topPadding = computed(() => renderStart.value * lineHeight)
  const remainingLines = computed(() => (count() - renderCount.value - renderStart.value))
  const bottomPadding = computed(() => remainingLines.value * lineHeight)
  function update(top: number, height: number) {
    const start = inBounds(Math.floor(top / lineHeight))
    const body = Math.ceil(height / lineHeight)
    const end = inBounds(start + body)

    const dangerStart = state.startIndex + dangerPadding
    const dangerEnd = state.endIndex - dangerPadding

    if (dangerStart >= start || dangerEnd <= end) {
      // reset bounds
      state.startIndex = start - linePadding - dangerPadding
      state.endIndex = start + body + linePadding + dangerPadding
    }
  }

  return {
    renderStart, renderCount,
    topPadding, bottomPadding,
    getIndex,
    update
  }
}