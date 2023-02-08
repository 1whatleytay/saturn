<template>
  <!-- Cursor Indicator -->
  <div
    class="w-0.5 bg-orange-400 hidden peer-focus:block absolute"
    :style="{
      left: `${props.position.offsetX}px`,
      top: `${props.position.offsetY}px`,
      height: `${props.lineHeight}px`,
      marginLeft: `-${props.cursorShift}px`
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
  cursorShift?: number
}>(), { lineHeight: 24, cursorShift: 1.3 })
</script>
