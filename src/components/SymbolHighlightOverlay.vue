<template>
  <div v-if="highlightIndices">
    <div
      v-for="matches in highlightIndices"
      class="absolute -z-10"
      :style="{
        top: `${matches.height}px`,
      }"
    >
      <div
        v-for="match in matches.matches"
        class="bg-slate-800 h-6 absolute rounded"
        :style="{
          left: `${match.offset}px`,
          width: `${match.size}px`,
        }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { symbolHighlights } from '../state/state'
import { computed } from 'vue'
import { SymbolHighlight } from '../utils/symbol-highlight'

const props = withDefaults(
  defineProps<{
    count: number
    start: number
    lineHeight?: number
  }>(),
  { lineHeight: 24 }
)

const highlightIndices = computed(() => {
  const pairs = [] as { height: number; matches: SymbolHighlight[] }[]

  const highlights = symbolHighlights(props.start, props.count)

  if (!highlights) {
    return
  }

  for (let a = 0; a < props.count; a++) {
    const line = a + props.start

    if (a >= highlights.length) {
      continue
    }

    const matches = highlights[a]
    if (!matches.length) {
      continue
    }

    pairs.push({
      height: props.lineHeight * line,
      matches,
    })
  }

  return pairs
})
</script>
