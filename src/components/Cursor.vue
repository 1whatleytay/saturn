<template>
  <!-- Cursor Indicator -->
  <div
    class="w-0.5 h-6 bg-orange-400 hidden peer-focus:block absolute mx-[-0.08rem]"
    :style="{
      left: `${props.position.offsetX}px`,
      top: `${props.position.offsetY}px`
    }"
  />
</template>

<script setup lang="ts">
import { CursorPosition } from '../utils/cursor'

const props = withDefaults(defineProps<{
  // virtualization props
  start: number
  count: number
  position: CursorPosition
  lineHeight?: number
}>(), { lineHeight: 24 })
</script>
