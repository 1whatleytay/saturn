<template>
  <button
    class="hover:bg-neutral-800 text-slate-300 flex items-center transition-[border-color] duration-200 space-x-4 h-10 px-6 text-xs font-medium border-b-2"
    :class="{
      'border-orange-400 bg-neutral-800': props.selected,
      'border-transparent': !props.selected,
    }"
  >
    <span class="max-w-[260px] truncate" :class="{'text-red-400': props.removed}">
      {{ props.title }}
    </span>

    <button
      v-if="props.deletable"
      @click.stop="emit('delete')"
      class="ml-3 translate-x-1 w-4 h-4 hover:bg-orange-400 text-lg p-0.5 hover:text-black rounded-full text-center group"
    >
      <span
        class="w-2 h-2 block rounded-full bg-gray-300 mx-auto"
        :class="{ 'block group-hover:hidden': marked, hidden: !marked }"
      />
      <XMarkIcon
        class="w-3 h-3"
        :class="{ 'hidden group-hover:block': marked }"
      />
    </button>
  </button>
</template>

<script setup lang="ts">
import { XMarkIcon } from '@heroicons/vue/20/solid'

const props = withDefaults(
  defineProps<{
    title: string
    removed?: boolean
    selected?: boolean
    deletable?: boolean
    marked?: boolean
  }>(),
  {
    selected: false,
    removed: false,
    deletable: false,
    marked: false,
  }
)

const emit = defineEmits(['delete'])
</script>
