<template>
  <div
    class="w-28 py-1 px-0.5 h-12"
    :class="!props.editable ? 'opacity-50' : ''"
  >
    <div class="text-xs pl-2 group" :class="props.classes">
      {{ props.name }}
    </div>

    <NumberField
      v-if="props.value !== undefined"
      :classes="`
              pl-2 py-1 rounded
              hover:bg-neutral-800 select-all cursor-text
              w-28 bg-transparent
              text-sm
              ${props.marked ? 'text-orange-200' : 'text-gray-100'}`"
      :model-value="props.value"
      @update:model-value="value => emit('set', value)"
      :checker="checker"
      :editable="props.editable"
      :hex="props.format === RegisterFormat.Hexadecimal"
      :signed="props.format === RegisterFormat.Signed"
    />
  </div>
</template>

<script setup lang="ts">
import { RegisterFormat } from '../../utils/settings'

import NumberField from './NumberField.vue'

const props = withDefaults(defineProps<{
  editable?: boolean,
  name: string,
  format?: RegisterFormat,
  value?: number,
  marked?: boolean,
  classes?: string
}>(), {
  editable: true,
  format: RegisterFormat.Hexadecimal,
  marked: false,
  classes: ''
})

const emit = defineEmits(['set'])

function checker(value: number): string | null {
  const valid = value >= 0 && value <= 0xffffffff

  return !valid ? 'Value must be between 0x0 and 0xffffffff' : null
}
</script>
