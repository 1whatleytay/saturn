<template>
  <span class="relative group">
    <input
      type="text"
      class="text-xs font-mono bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
      spellcheck="false"
      :class="{
        'ring-2 ring-red-500': state.error !== null
      }"
      v-model="state.value"
    />

    <span v-if="state.error" class="
      absolute top-6 hidden group-hover:block
      py-2 px-4 w-auto
      bg-neutral-900 rounded
      shadow-xl
      absolute z-30
      text-red-400 font-medium font-sans
      hidden group-hover:block">
      {{ state.error }}
    </span>
  </span>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'

const props = withDefaults(defineProps<{
  modelValue: number,
  hex?: boolean,
  checker?: (value: number) => string | null
}>(), {
  hex: false
})

const emit = defineEmits(['update:modelValue'])

const state = reactive({
  expected: props.modelValue,
  value: formatHex(props.modelValue, props.hex),
  error: null as string | null,
})

watch(() => props.modelValue, value => {
  if (value !== state.expected) {
    state.expected = value
    state.value = formatHex(props.modelValue, props.hex)
  }
})

function parse(text: string): number | null {
  let result: number

  if (text.startsWith('0x')) {
    const rest = text.substring(2)

    // Validate
    if (!/^[0-9a-f]*$/.test(rest)) {
      return null
    }

    result = parseInt(rest, 16)
  } else {
    // Validate
    if (!/^[0-9]*$/.test(text)) {
      return null
    }

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

watch(() => state.value, value => {
  const result = parse(value)

  if (result) {
    if (props.checker) {
      state.error = props.checker(result)
    } else {
      state.error = null
    }

    if (state.error === null) {
      state.expected = result
      emit('update:modelValue', result)
    }
  } else {
    state.error = 'This field only accepts numerical values'
  }
})
</script>
