<template>
  <button
    class="text-neutral-300 px-4 py-3 rounded-lg flex items-center transition-colors"
    :class="{ [props.classes]: true, 'bg-neutral-800': !props.modelValue, 'bg-slate-800': props.modelValue }"
    @click="emit('update:modelValue', !props.modelValue)"
  >
    <span
      class="w-4 h-4 block rounded ring-2 ring-neutral-600 mr-4 flex items-center justify-center"
      :class="{'animate-bump ring-slate-600': props.modelValue}"
    >
      <CheckIcon v-if="props.modelValue" class="text-gray-300 font-bold w-3 h-3" />
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
