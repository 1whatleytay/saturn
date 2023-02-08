<template>
  <div
    v-if="highlights.state.highlight"
    class="absolute h-6 border-b-2 border-red-500 bg-red-500 bg-opacity-25 group"
    :style="{
      top: `${props.lineHeight * highlights.state.highlight.line}px`,
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
</template>

<script setup lang="ts">
import { highlights } from '../state/state'

const props = withDefaults(defineProps<{
  lineHeight?: number
}>(), { lineHeight: 24 })
</script>
