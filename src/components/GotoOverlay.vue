<template>
  <div
    class="absolute h-6 border-b-2 border-dotted cursor-pointer"
    :style="{
      top: `${props.lineHeight * props.highlight.line}px`,
      left: `${props.highlight.offset}px`,
      width: `${props.highlight.size}px`
    }"
  >
    <div v-if="props.highlight.message" class="
      mt-6 py-2 px-4 w-auto
      bg-neutral-700 rounded
      shadow-xl
      absolute z-30
      text-gray-300 font-medium font-sans
      flex items-center
    ">
      Jump to
      <span class="text-gray-200 font-bold font-mono">{{ props.highlight.message.label }}</span>
      (line {{ props.highlight.message.line + 1 }})

      <div
        v-if="props.highlight.message.type"
        class="rounded ml-4 w-4 h-4 text-black flex items-center justify-center text-xs shrink-0"
        :class="[suggestionStyle(props.highlight.message.type)]"
      >
        {{ suggestionLetter(props.highlight.message.type) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Highlights } from '../utils/highlights'
import { UnwrapRef } from 'vue'
import { GotoMessage } from '../utils/goto'
import { suggestionLetter, suggestionStyle } from '../utils/query/suggestion-styles'

const props = withDefaults(defineProps<{
  highlight: Highlights<UnwrapRef<GotoMessage>>,
  lineHeight?: number
}>(), { lineHeight: 24 })
</script>
