<template>
  <div class="pointer-events-none absolute top-0">
    <div class="absolute" :style="{ top: `${props.range.top}px` }">
      <div
        v-for="(text, index) in props.range.ranges"
        :key="index"
        class="flex items-center"
        :style="{ height: `${props.lineHeight}px` }"
      >
        <span class="opacity-0">
          {{ text.leading }}
        </span>

        <span class="-mx-0.5 rounded bg-blue-500 px-0.5 opacity-30">
          <span class="opacity-0">
            {{ text.body }}
          </span>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RangeSelection } from '../utils/cursor'

const props = withDefaults(
  defineProps<{
    range: RangeSelection
    lineHeight?: number
  }>(),
  { lineHeight: 24 },
)
</script>
