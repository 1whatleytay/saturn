<template>
  <div
    v-if="suggestions.state.results.length"
    class="w-80 h-40 text-sm font-mono overflow-hidden rounded-lg mt-6 p-2 bg-neutral-900 border border-neutral-800 absolute mx-[-0.08rem] z-20"
    :style="{
      left: `${position.offsetX}px`,
      top: `${position.offsetY}px`,
    }"
    @wheel.prevent.stop="scrollSuggestions"
    ref="suggestionsParent"
  >
    <div
      ref="suggestionsPane"
      :style="{ marginTop: `-${suggestionsScroll}px` }"
      @mousedown.stop.prevent
    >
      <div
        v-for="(suggestion, index) in suggestions.state.results"
        :key="index"
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
              {{
                suggestion.replace.substring(
                  suggestion.range.start,
                  suggestion.range.end + 1
                )
              }}
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
          <span
            v-if="suggestion.name"
            class="text-neutral-500 mr-2 text-xs max-w-[12rem] shrink truncate"
          >
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
import { applyMergeSuggestion, position, suggestions } from '../state/state'
import { ref, watch } from 'vue'
import {
  suggestionLetter,
  suggestionStyle,
} from '../utils/query/suggestion-styles'

const props = withDefaults(
  defineProps<{
    lineHeight?: number
  }>(),
  { lineHeight: 24 }
)

const suggestionsScroll = ref(0)

const suggestionsPane = ref(null as HTMLElement | null)
const suggestionsParent = ref(null as HTMLElement | null)

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
    const bottom =
      suggestionsPane.value.clientHeight -
      suggestionsParent.value.clientHeight +
      16

    if (bottom >= 0) {
      result = Math.min(result, bottom)
    }
  }

  suggestionsScroll.value = result
}

function scrollSuggestions(event: WheelEvent) {
  scrollSuggestionsTo(suggestionsScroll.value + event.deltaY)
}

watch(
  () => suggestions.state.results,
  () => {
    scrollSuggestionsTo(0)
  }
)

watch(
  () => suggestions.state.index,
  (value) => {
    const top = value * props.lineHeight
    const bottom = top + props.lineHeight

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
  }
)
</script>
