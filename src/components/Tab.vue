<template>
  <button @click="emit('select')" class="
        hover:bg-neutral-800
        text-slate-300
        flex items-center
        transition-[border-color]
        duration-200
        space-x-4
        h-10
        px-6
        text-xs
        font-bold
        max-w-xs
        border-b-2
      " :class="{
        'border-orange-400 bg-neutral-800': selected ?? false,
        'border-transparent': !(selected ?? false),
      }">
    {{ tab.title }}

    <button
      @click.stop="emit('delete')"
      class="ml-3 hover:bg-orange-400 text-lg p-0.5 hover:text-black rounded-full text-center"
    >
      <XMarkIcon class="w-3 h-3" />
    </button>
  </button>
</template>

<script setup lang="ts">
import { XMarkIcon } from '@heroicons/vue/20/solid'

import { EditorTab } from '../state/editor-state'

const { tab, selected } = defineProps<{
  tab: EditorTab,
  selected?: boolean
}>()

const emit = defineEmits(['select', 'delete'])
</script>
