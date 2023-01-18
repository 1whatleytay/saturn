<template>
  <button
    @click="emit('select')"
    class="
      hover:bg-neutral-800
      text-slate-300
      flex items-center
      transition-[border-color]
      duration-200
      space-x-4
      h-10
      px-6
      text-xs
      font-medium
      max-w-xs
      border-b-2
    "
    :class="{
      'border-orange-400 bg-neutral-800': props.selected,
      'border-transparent': !props.selected,
    }"
  >
    {{ props.title }}

    <button
      v-if="props.deletable"
      @click.stop="emit('delete')"
      class="ml-3 w-4 h-4 hover:bg-orange-400 text-lg p-0.5 hover:text-black rounded-full text-center group"
    >
      <PencilIcon class="w-2 h-2" :class="{ 'block group-hover:hidden': marked, 'hidden': !marked }" />
      <XMarkIcon class="w-3 h-3" :class="{ 'hidden group-hover:block': marked }" />
    </button>
  </button>
</template>

<script setup lang="ts">
import { XMarkIcon, PencilIcon } from '@heroicons/vue/20/solid'

const props = withDefaults(defineProps<{
  title: string,
  selected?: boolean,
  deletable?: boolean,
  marked?: boolean
}>(), {
  selected: false,
  deletable: false,
  marked: false
})

const emit = defineEmits(['select', 'delete'])
</script>
