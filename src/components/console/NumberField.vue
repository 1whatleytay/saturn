<template>
  <span class="group relative inline-block">
    <input
      type="text"
      class="rounded bg-neutral-300 px-2 py-1 font-mono text-neutral-800 dark:bg-neutral-800 dark:text-neutral-300"
      spellcheck="false"
      :class="[
        state.error !== null ? 'ring-2 ring-red-500' : '',
        props.classes,
      ]"
      v-model="state.value"
      @change="clean"
      :disabled="!props.editable"
      @keydown.enter="clean"
    />

    <span
      v-if="state.error"
      class="absolute top-6 z-30 hidden w-80 rounded bg-neutral-200 px-4 py-2 font-sans font-medium text-red-400 shadow-xl group-hover:block dark:bg-neutral-900"
    >
      {{ state.error }}
    </span>
  </span>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue: number
    hex?: boolean
    signed?: boolean
    classes?: string
    checker?: (value: number) => string | null
    editable?: boolean
    cleanOnly?: boolean
    bytes?: number
  }>(),
  {
    hex: false,
    signed: false,
    classes: '',
    editable: true,
    cleanOnly: true,
    bytes: 4,
  },
)

const emit = defineEmits(['update:modelValue'])

const state = reactive({
  expected: props.modelValue,
  value: formatHex(props.modelValue, props.hex, props.signed),
  error: null as string | null,
})

let debounce = null as number | null

function clean() {
  if (debounce) {
    window.clearTimeout(debounce)
  }

  if (props.cleanOnly) {
    emit('update:modelValue', state.expected)
  }

  state.value = formatHex(state.expected, props.hex, props.signed)
}

watch(
  () => props.modelValue,
  (value) => {
    if (value !== state.expected) {
      state.expected = value
      state.value = formatHex(props.modelValue, props.hex, props.signed)
    }
  },
)

watch(
  () => [props.hex, props.signed],
  (value) => {
    state.value = formatHex(props.modelValue, value[0], value[1])
  },
)

function parse(leading: string): number | null {
  let result: number

  const hasSign = (leading.length && leading[0] === '+') || leading[0] === '-'
  const text = hasSign ? leading.substring(1) : leading
  const negative = hasSign && leading[0] === '-'

  if (text.startsWith('0x')) {
    const rest = text.substring(2)

    // Validate
    if (!/^[0-9a-fA-F]*$/.test(rest)) {
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

  if (negative) {
    result = 0x100000000 - result
  }

  if (props.bytes < 4) {
    result = result & (~0 >> ((4 - props.bytes) * 8))
  }

  return isNaN(result) ? null : result
}

function formatHex(value: number, hex: boolean, signed: boolean): string {
  if (hex) {
    return `0x${value.toString(16)}`
  }

  const ext = ~0 << (props.bytes * 8)

  if (signed) {
    const sign = (value & (0x80000000 >> ((4 - props.bytes) * 8))) != 0

    if (sign && props.bytes < 4) {
      value |= ext
    }

    return (value >> 0).toString()
  }

  return value.toString()
}

watch(
  () => state.value,
  (value) => {
    const result = parse(value)

    if (result !== null) {
      if (props.checker) {
        state.error = props.checker(result)
      } else {
        state.error = null
      }

      if (state.error === null) {
        state.expected = result

        if (debounce) {
          window.clearTimeout(debounce)
        }

        if (!props.cleanOnly) {
          emit('update:modelValue', result)
        } else {
          debounce = window.setTimeout(() => {
            emit('update:modelValue', result)
          }, 500)
        }
      }
    } else {
      state.error = 'This field only accepts numerical values'
    }
  },
)
</script>
