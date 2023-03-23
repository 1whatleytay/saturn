<template>
  <span class="inline-block relative group">
    <input
      type="text"
      class="font-mono bg-neutral-800 text-neutral-300 px-2 py-1 rounded"
      spellcheck="false"
      :class="[
        state.error !== null ? 'ring-2 ring-red-500' : '',
        props.classes
      ]"
      v-model="state.value"
      @change="clean"
      :disabled="!props.editable"
      @keydown.enter="clean"
    />

    <span v-if="state.error" class="
      absolute top-6 hidden group-hover:block
      py-2 px-4 w-auto
      bg-neutral-900 rounded
      shadow-xl
      absolute z-30
      w-80
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
  classes?: string
  checker?: (value: number) => string | null,
  editable?: boolean,
  cleanOnly?: boolean
}>(), {
  hex: false,
  classes: '',
  editable: true,
  cleanOnly: true
})

const emit = defineEmits(['update:modelValue'])

const state = reactive({
  expected: props.modelValue,
  value: formatHex(props.modelValue, props.hex),
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

  state.value = formatHex(state.expected, props.hex)
}

watch(() => props.modelValue, value => {
  if (value !== state.expected) {
    state.expected = value
    state.value = formatHex(props.modelValue, props.hex)
  }
})

watch(() => props.hex, value => {
  state.value = formatHex(props.modelValue, value)
})

function parse(leading: string): number | null {
  let result: number

  const hasSign = leading.length && leading[0] === '+' || leading[0] === '-'
  const text = hasSign ? leading.substring(1) : leading
  const negative = hasSign && leading[0] === '-'

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

  if (negative) {
    result = 0x100000000 - result
  }

  return isNaN(result) ? null : result
}

function formatHex(value: number, hex: boolean): string {
  if (hex) {
    const signed = Math.abs(value).toString(16)

    return `${value < 0 ? '-' : ''}0x${signed}`
  } else {
    return value.toString()
  }
}

watch(() => state.value, value => {
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
})
</script>
