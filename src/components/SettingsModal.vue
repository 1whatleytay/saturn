<template>
  <Modal :show="props.show" @close="emit('close')">
    <div
      class="max-w-2xl bg-neutral-900 rounded-xl px-8 py-6 mx-auto flex flex-col shadow pointer-events-auto"
    >
      <div class="text-lg font-medium flex items-center">
        <CogIcon class="w-5 h-5 mr-2" /> Settings

        <button
          class="w-8 h-8 ml-auto rounded hover:bg-slate-800 text-slate-300 shrink-0 flex items-center justify-center"
          @click="emit('close')"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>

      <div class="mt-8">
        <div class="font-bold uppercase text-sm">
          Font Size
        </div>

        <div class="text-gray-300 text-sm mt-1">
          Adjust the size of the text in the editor window.
        </div>

        <NumberField class="my-2 text-sm" v-model="settings.editor.tabSize" />
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { CogIcon, XMarkIcon } from '@heroicons/vue/24/solid'
import Modal from './Modal.vue'
import NumberField from './console/NumberField.vue'

import { settings } from '../state/state'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits(['close'])
</script>
