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
</template>

<script setup lang="ts">
import { cursor, lines, selectionRange } from '../utils/editor-cursor'
import { computed } from 'vue'
import { regular } from '../utils/text-size'

interface LineRange {
  leading: string,
  body: string
}

interface RangeSelection {
  ranges: LineRange[],
  top: number
}

const range = computed((): RangeSelection | null => {
  const range = selectionRange()

  if (!range) {
    return null
  }

  const all = lines.value

  const { height } = regular.calculate('')
  const top = range.startLine * height

  if (range.startLine == range.endLine) {
    const line = all[range.startLine]

    const ranges = [{
      leading: line.substring(0, range.startIndex),
      body: line.substring(range.startIndex, range.endIndex)
    }]

    return { ranges, top }
  }

  const result = [] as LineRange[]

  const first = all[range.startLine]
  result.push({
    leading: first.substring(0, range.startIndex),
    body: first.substring(range.startIndex)
  })

  for (let a = range.startLine + 1; a < range.endLine; a++) {
    result.push({
      leading: '',
      body: all[a]
    })
  }

  const last = all[range.endLine]
  result.push({
    leading: '',
    body: last.substring(0, range.endIndex)
  })

  return { top, ranges: result }
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
