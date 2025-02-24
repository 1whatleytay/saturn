<template>
  <button
    class="flex items-center rounded-lg px-4 py-3 text-neutral-800 transition-colors dark:text-neutral-300"
    :class="{
      [props.classes]: true,
      'bg-neutral-300 dark:bg-neutral-800': !props.modelValue,
      'bg-slate-300 dark:bg-slate-800': props.modelValue,
    }"
    @click="emit('update:modelValue', !props.modelValue)"
  >
    <span
      class="mr-4 block flex h-4 w-4 items-center justify-center rounded ring-2 ring-neutral-600"
      :class="{ 'animate-bump ring-slate-600': props.modelValue }"
    >
      <CheckIcon
        v-if="props.modelValue"
        class="h-3 w-3 font-bold text-gray-800 dark:text-gray-300"
      />
    </span>

    <span class="text-sm font-bold uppercase">
      {{ props.title }}
    </span>
  </button>
</template>

<script setup lang="ts">
import { CheckIcon } from '@heroicons/vue/24/solid'

const props = withDefaults(
  defineProps<{
    modelValue: boolean
    title: string
    classes?: string
  }>(),
  {
    classes: '',
  },
)

const emit = defineEmits(['update:modelValue'])
</script>
