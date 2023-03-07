<template>
  <div class="absolute top-0 pointer-events-none">
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

        <span class="rounded opacity-30 px-0.5 -mx-0.5 bg-blue-500">
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

const props = withDefaults(defineProps<{
  range: RangeSelection
  lineHeight?: number
}>(), { lineHeight: 24 })
</script>
