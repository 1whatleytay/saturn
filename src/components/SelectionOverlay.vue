<template>
  <div class="absolute top-0 pointer-events-none">
    <div class="absolute" :style="{ top: `${props.range.top}px` }">
      <div
        v-for="(text, index) in props.range.ranges"
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
</template>

<script setup lang="ts">
import { RangeSelection } from '../utils/cursor'

const props = defineProps<{
  range: RangeSelection
}>()
</script>
