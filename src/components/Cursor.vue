<template>
  <!-- Selection -->
  <div class="absolute top-0 pointer-events-none" v-if="range">
    <div class="absolute" :style="{ top: `${range.top}px` }">
      <div
        v-for="(text, index) in range.ranges"
        :key="index"
        class="h-6 flex items-center"
      >
        <span class="opacity-0">
          {{ text.leading }}
        </span>

        <span class="rounded opacity-30 px-1 -mx-1 bg-blue-500">
          <span class="opacity-0">
            {{ text.body }}
          </span>
        </span>
      </div>
    </div>
  </div>

  <!-- Error Highlights -->
  <div
    v-if="highlights.state.highlight"
    class="absolute h-6 border-b-2 border-red-500 bg-red-500 bg-opacity-25 group"
    :style="{
      top: `${lineHeight * highlights.state.highlight.line}px`,
      left: `${highlights.state.highlight.offset}px`,
      width: `${highlights.state.highlight.size}px`
    }"
  >
    <div v-if="highlights.state.highlight.message" class="
      mt-6 py-2 px-4 w-auto
      bg-neutral-900 rounded
      shadow-xl
      absolute z-30
      text-red-400 font-medium font-sans
      hidden group-hover:block"
    >
      {{ highlights.state.highlight.message }}
    </div>
  </div>

  <!-- Find Results -->
  <div v-if="find.state.show">
    <div
      v-for="matches in findIndices"
      class="absolute"
      :style="{
        top: `${matches.height}px`,
      }"
    >
      <div
        v-for="match in matches.matches"
        class="bg-yellow-500 h-6 bg-opacity-30 absolute"
        :class="{ 'bg-opacity-50': match === find.state.lastMatch }"
        :style="{
          left: `${match.offset}px`,
          width: `${match.size}px`
        }"
      />
    </div>
  </div>

  <!-- Cursor Indicator -->
  <div
    class="w-0.5 h-6 bg-orange-400 hidden peer-focus:block absolute mx-[-0.08rem]"
    :style="{
      left: `${position.offsetX}px`,
      top: `${position.offsetY}px`
    }"
  />

  <!-- Suggestions -->
  <div
    v-if="suggestions.state.results.length"
    class="
      w-80 h-40
      text-sm font-mono
      overflow-clip
      rounded-lg
      mt-6 p-2
      overflow-y-clip
      bg-neutral-900 border border-neutral-800
      absolute mx-[-0.08rem]
    " :style="{
      left: `${position.offsetX}px`,
      top: `${position.offsetY}px`,
    }"
    @wheel="scrollSuggestions"
    ref="suggestionsParent"
  >
    <div
      ref="suggestionsPane"
      :style="{ marginTop: `-${suggestionsScroll}px` }"
      @mousedown.stop.prevent
    >
      <div
        v-for="(suggestion, index) in suggestions.state.results"
        :key="suggestion.replace"
        class="w-full h-6 rounded px-2 flex items-center cursor-pointer transition-colors duration-150"
        :class="{ 'bg-neutral-700': index === suggestions.state.index }"
        @click.stop="suggestions.state.index = index"
        @dblclick.stop="merge(index)"
      >
        <span class="truncate">
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
            {{ suggestion.replace }}
          </span>
        </span>

        <div class="ml-auto flex items-center">
           <span v-if="suggestion.name" class="text-neutral-500 mr-2 text-xs max-w-[12rem] shrink truncate">
             {{ suggestion.name }}
           </span>

          <div
            class="rounded w-4 h-4 text-black flex items-center justify-center text-xs shrink-0"
            :class="[suggestionStyle(suggestion.type)]"
          >
            {{ suggestionLetter(suggestion.type) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { applyMergeSuggestion, highlights, position, suggestions, find } from '../state/state'
import { computed, ref, watch } from 'vue'
import { regular } from '../utils/query/text-size'
import { SuggestionType } from '../utils/languages/suggestions'
import { selectionRange, tab, tabBody } from '../state/tabs-state'
import { FindMatch } from '../utils/find'

const suggestionsScroll = ref(0)
const suggestionsPane = ref(null as HTMLElement | null)
const suggestionsParent = ref(null as HTMLElement | null)

const lineHeight = 24 // h-6

const findIndices = computed(() => {
  const pairs = [] as { height: number, matches: FindMatch[] }[]

  for (let a = 0; a < props.count; a++) {
    const line = a + props.start
    if (line < 0 || line >= find.state.matches.length) {
      continue
    }

    const matches = find.state.matches[line]
    if (!matches.length) {
      continue
    }

    pairs.push({
      height: lineHeight * line,
      matches
    })
  }

  return pairs
})

function merge(index: number) {
  suggestions.state.index = index

  const suggestion = suggestions.mergeSuggestion()

  if (suggestion) {
    applyMergeSuggestion(suggestion)
  }
}

function scrollSuggestionsTo(point: number) {
  let result = Math.max(point, 0)

  if (suggestionsPane.value && suggestionsParent.value) {
    const bottom = suggestionsPane.value.clientHeight - suggestionsParent.value.clientHeight + 16

    if (bottom >= 0) {
      result = Math.min(result, bottom)
    }
  }

  suggestionsScroll.value = result
}

function scrollSuggestions(event: WheelEvent) {
  scrollSuggestionsTo(suggestionsScroll.value + event.deltaY)
}

watch(() => suggestions.state.results, () => {
  scrollSuggestionsTo(0)
})

watch(() => suggestions.state.index, value => {
  const top = value * lineHeight
  const bottom = top + lineHeight

  if (!suggestionsParent.value) {
    return
  }

  const parentHeight = suggestionsParent.value.clientHeight
  const start = suggestionsScroll.value
  const end = start + parentHeight

  if (top < start) {
    scrollSuggestionsTo(top)
  } else if (bottom > end) {
    scrollSuggestionsTo(bottom - parentHeight + 16)
  }
})

function suggestionLetter(type?: SuggestionType): string {
  switch (type) {
    case SuggestionType.Instruction:
      return 'I'

    case SuggestionType.Register:
      return 'R'

    case SuggestionType.Directive:
      return 'D'

    default:
      return 'O'
  }
}

function suggestionStyle(type?: SuggestionType): string {
  switch (type) {
    case SuggestionType.Instruction:
      return 'bg-cyan-500 text-cyan-900'

    case SuggestionType.Register:
      return 'bg-orange-300 text-orange-800'

    case SuggestionType.Directive:
      return 'bg-red-300 text-red-800'

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
  const current = tab()

  if (!current) {
    return null
  }

  const range = selectionRange(current.cursor)

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
