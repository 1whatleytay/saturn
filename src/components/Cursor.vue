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
      overflow-y-clip
      bg-neutral-900 border border-neutral-800
      absolute mx-[-0.08rem]
    " :style="{
      left: `${cursor.offsetX}px`,
      top: `${cursor.offsetY}px`,
    }"
    @wheel="scrollSuggestions"
  >
    <div :style="{ marginTop: `-${suggestionsScroll}px` }">
      <div
        v-for="(suggestion, index) in suggestions.results"
        :key="suggestion.replace"
        class="w-full h-6 rounded px-2 flex items-center"
        :class="{ 'bg-neutral-700': index === suggestions.index }"
      >
        <!-- Ignoring Name for now while I figure out what replace does... -->
        <span v-if="suggestion.range">
          <span>
            {{ suggestion.replace.substring(0, suggestion.range.start) }}
          </span>

          <span class="font-bold">
            {{ suggestion.replace.substring(suggestion.range.start, suggestion.range.end + 1) }}
          </span>

          <span>
            {{ suggestion.replace.substring(suggestion.range.end + 1) }}
          </span>
        </span>

        <span v-else>
          {{ suggestion.name }}
        </span>

        <div
          class="ml-auto rounded w-4 h-4 text-black flex items-center justify-center text-xs"
          :class="[suggestionStyle(suggestion.type)]"
        >
          {{ suggestionLetter(suggestion.type) }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { cursor, selectionRange, suggestions, tabBody } from '../state/cursor-state'
import { computed, ref } from 'vue'
import { regular } from '../utils/text-size'
import { Suggestion, SuggestionType } from '../utils/languages/suggestions'

const suggestionsScroll = ref(0)

function scrollSuggestions(event: WheelEvent) {
  suggestionsScroll.value = Math.max(suggestionsScroll.value + event.deltaY, 0)
}

function suggestionLetter(type: SuggestionType): string {
  switch (type) {
    case SuggestionType.Instruction:
      return 'I'

    default:
      return 'O'
  }
}

function suggestionStyle(type: SuggestionType): string {
  switch (type) {
    case SuggestionType.Instruction:
      return 'bg-cyan-500 text-cyan-900'

    default:
      return 'bg-gray-700'
  }
}

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
