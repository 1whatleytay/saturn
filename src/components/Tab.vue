<template>
  <button
    class="flex h-10 items-center space-x-4 border-b-2 px-6 text-xs font-medium text-slate-800 transition-[border-color] duration-200 hover:bg-neutral-300 dark:text-slate-300 dark:hover:bg-neutral-700"
    :class="{
      'border-orange-400 dark:bg-neutral-800': props.selected,
      'border-transparent': !props.selected,
    }"
  >
    <span
      class="max-w-[260px] truncate"
      :class="{ 'text-red-400': props.removed }"
    >
      {{ props.title }}
    </span>

    <button
      v-if="props.deletable"
      @click.stop="emit('delete')"
      class="group ml-3 h-4 w-4 translate-x-1 rounded-full p-0.5 text-center text-lg hover:bg-orange-400 hover:text-black"
    >
      <span
        class="mx-auto block h-2 w-2 rounded-full bg-gray-700 dark:bg-gray-300"
        :class="{ 'block group-hover:hidden': marked, hidden: !marked }"
      />
      <XMarkIcon
        class="h-3 w-3"
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
  },
)

const emit = defineEmits(['delete'])
</script>
