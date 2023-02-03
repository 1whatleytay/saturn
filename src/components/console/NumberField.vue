<template>
  <input
    type="text"
    class="text-xs font-mono bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
    spellcheck="false"
    :class="{
      '': state.valid,
      'ring-2 ring-red-500': !state.valid
    }"
    v-model="state.value"
  />
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'

const props = withDefaults(defineProps<{
  modelValue: number,
  hex?: boolean
}>(), {
  hex: false
})

function parse(text: string): number | null {
  let result: number

  if (text.startsWith('0x')) {
    result = parseInt(text.substring(2), 16)
  } else {
    result = parseInt(text)
  }

  return isNaN(result) ? null : result
}

function formatHex(value: number, hex: boolean): string {
  if (hex) {
    return `0x${value.toString(16)}`
  } else {
    return value.toString()
  }
}

const emit = defineEmits(['update:modelValue'])

const state = reactive({
  value: formatHex(props.modelValue, props.hex),
  valid: true
})

watch(() => state.value, value => {
  const result = parse(value)

  state.valid = result !== null

  if (result) {
    emit('update:modelValue', result)
  }
})
</script>
