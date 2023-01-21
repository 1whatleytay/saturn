<template>
  <div class="absolute top-0 pointer-events-none" v-if="range">
    <div class="absolute" :style="{ top: `${range.top}px` }">
      <div
        v-for="(text, index) in range.ranges"
        :key="index"
        class="line"
      >
        <span class="opacity-0">
          {{ text.leading }}
        </span>

        <span class="selection">
          <span class="opacity-0">
            {{ text.body }}
          </span>
        </span>
      </div>
    </div>
  </div>

  <div
    class="w-0.5 h-6 bg-orange-400 hidden peer-focus:block absolute mx-[-0.08rem]"
    :style="{
      left: `${cursor.offsetX}px`,
      top: `${cursor.offsetY}px`
    }"
  />

  <div
    v-if="suggestions.results.length"
    class="
      w-48 h-32
      text-sm font-mono
      overflow-clip
      rounded-lg
      mt-6 p-2
      overflow-y-auto
      bg-neutral-900 border border-neutral-800
      absolute mx-[-0.08rem]
    " :style="{
      left: `${cursor.offsetX}px`,
      top: `${cursor.offsetY}px`
    }"
  >
    <div
      v-for="(suggestion, index) in suggestions.results"
      :key="suggestion.replace"
      class="w-full h-6 rounded"
      :class="{ 'bg-neutral-700': index === suggestions.index }"
    >
      {{ suggestion.name }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { cursor, tabBody, selectionRange, suggestions } from '../state/cursor-state'
import { computed } from 'vue'
import { regular } from '../utils/text-size'

const props = defineProps<{
  // virtualization props
  start: number,
  count: number,
}>()

interface LineRange {
  leading: string,
  body: string
}

interface RangeSelection {
  ranges: LineRange[],
  top: number
}

function inBounds(line: number): boolean {
  return props.start <= line && line < props.start + props.count
}

const range = computed((): RangeSelection | null => {
  const range = selectionRange()

  if (!range) {
    return null
  }

  const all = tabBody.value

  const { height } = regular.calculate('')
  let top = null as number | null
  const suggestTop = (line: number) => {
    if (top === null) {
      top = line * height
    }
  }

  if (range.startLine == range.endLine) {
    if (!inBounds(range.startLine)) {
      return null // ?
    }

    const line = all[range.startLine]

    const ranges = [{
      leading: line.substring(0, range.startIndex),
      body: line.substring(range.startIndex, range.endIndex)
    }]

    return { ranges, top: range.startLine * height }
  }

  const result = [] as LineRange[]

  if (inBounds(range.startLine)) {
    suggestTop(range.startLine)
    const first = all[range.startLine]
    result.push({
      leading: first.substring(0, range.startIndex),
      body: first.substring(range.startIndex)
    })
  }

  // Just going to hope this intersection thingy works.
  const intersectionStart = Math.max(range.startLine + 1, props.start)
  const intersectionEnd = Math.min(range.endLine, props.start + props.count)

  if (intersectionStart < intersectionEnd) {
    suggestTop(intersectionStart)
  }

  for (let a = intersectionStart; a < intersectionEnd; a++) {
    result.push({
      leading: '',
      body: all[a]
    })
  }

  if (inBounds(range.endLine)) {
    suggestTop(range.endLine)
    const last = all[range.endLine]
    result.push({
      leading: '',
      body: last.substring(0, range.endIndex)
    })
  }

  return { top: top ?? range.startLine * height, ranges: result }
})
</script>

<style scoped>
.line {
  @apply h-6 flex items-center;
}

.selection {
  @apply rounded opacity-30 px-1 -mx-1 bg-blue-500;
}
</style>
