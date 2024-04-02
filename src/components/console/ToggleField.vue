<template>
  <button
    class="dark:text-neutral-300 text-neutral-800 px-4 py-3 rounded-lg flex items-center transition-colors"
    :class="{ [props.classes]: true, 'dark:bg-neutral-800 bg-neutral-300': !props.modelValue, 'dark:bg-slate-800 bg-slate-300': props.modelValue }"
    @click="emit('update:modelValue', !props.modelValue)"
  >
    <span
      class="w-4 h-4 block rounded ring-2 ring-neutral-600 mr-4 flex items-center justify-center"
      :class="{'animate-bump ring-slate-600': props.modelValue}"
    >
      <CheckIcon v-if="props.modelValue" class="dark:text-gray-300 text-gray-800 font-bold w-3 h-3" />
    </span>

    <span class="text-sm uppercase font-bold">
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
    classes: ''
  }
)

const emit = defineEmits(['update:modelValue'])
</script>
