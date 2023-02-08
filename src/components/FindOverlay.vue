<template>
  <div v-if="find.state.show">
    {{ findIndices.length }}
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
</template>

<script setup lang="ts">
import { FindMatch } from '../utils/find'
import { find } from '../state/state'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  count: number,
  start: number,
  lineHeight?: number
}>(), { lineHeight: 24 })

const findIndices = computed(() => {
  const pairs = [] as { height: number, matches: FindMatch[] }[]

  console.log({ count: props.count, start: props.start })
  for (let a = 0; a < props.count; a++) {
    const line = a + props.start
    console.log({ line })
    if (line < 0 || line >= find.state.matches.length) {
      continue
    }

    const matches = find.state.matches[line]
    if (!matches.length) {
      continue
    }

    console.log('adding one')
    pairs.push({
      height: props.lineHeight * line,
      matches
    })
  }

  console.log(pairs)

  return pairs
})
</script>
