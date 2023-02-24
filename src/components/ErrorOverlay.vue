<template>
  <div
    class="absolute h-6 border-b-2 border-red-500 bg-red-500 bg-opacity-25 group"
    :style="{
      top: `${props.lineHeight * props.highlight.line}px`,
      left: `${props.highlight.offset}px`,
      width: `${props.highlight.size}px`
    }"
  >
    <div v-if="props.highlight.message" class="
      mt-6 py-2 px-4 w-auto
      bg-neutral-900 rounded
      shadow-xl
      absolute z-30
      text-red-400 font-medium font-sans
      hidden group-hover:block"
    >
      {{ props.highlight.message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { Highlights } from '../utils/highlights'

const props = withDefaults(defineProps<{
  highlight: Highlights,
  lineHeight?: number
}>(), { lineHeight: 24 })
</script>
