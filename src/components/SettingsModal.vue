<template>
  <Modal :show="props.show" @close="emit('close')">
    <div
      class="max-w-2xl dark:bg-neutral-900 bg-neutral-200 rounded-xl px-8 py-6 mx-auto flex flex-col shadow pointer-events-auto overflow-y-scroll max-h-[84vh]"
    >
      <div class="text-2xl font-semibold flex items-center dark:bg-neutral-900 bg-neutral-200 w-full my-2 shrink-0">
        <CogIcon class="w-7 h-7 mr-3 shrink-0" /> Settings

        <button
          class="w-8 h-8 ml-auto rounded hover:bg-slate-800 dark:text-slate-300 text-slate-800 shrink-0 flex items-center justify-center"
          @click="emit('close')"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>

      <div class="mt-8">
        <div class="font-bold uppercase text-sm">
          Tab Size
        </div>

        <div class="dark:text-gray-300 text-gray-800 text-sm mt-1">
          Number of spaces that make up one tab (when the tab key is hit).
        </div>

        <NumberField class="my-2 text-sm" v-model="settings.editor.tabSize" />
      </div>

<!--      <div class="mt-8">-->
<!--        <div class="font-bold uppercase text-sm">-->
<!--          Font Size-->
<!--        </div>-->

<!--        <div class="dark:text-gray-300 text-gray-800 text-sm mt-1">-->
<!--          Adjust the size of the text in the editor window.-->
<!--        </div>-->

<!--        <NumberField class="my-2 text-sm" v-model="settings.editor.fontSize" />-->
<!--      </div>-->

<!--      <div class="mt-8">-->
<!--        <div class="font-bold uppercase text-sm">-->
<!--          Console Font Size-->
<!--        </div>-->

<!--        <div class="dark:text-gray-300 text-gray-800 text-sm mt-1">-->
<!--          Adjust the size of the text in the debug console.-->
<!--        </div>-->

<!--        <NumberField class="my-2 text-sm" v-model="settings.editor.consoleFontSize" />-->
<!--      </div>-->

      <div class="mt-8">
        <div class="font-bold uppercase text-sm">
          Time Travel
        </div>

        <div class="dark:text-gray-300 text-gray-800 text-sm mt-1">
          When enabled, allows fpr stepping backwards when running your program. Enabling may affect performance.
        </div>

        <ToggleField
          class="my-2"
          title="Use Time Travel"
          v-model="settings.execution.timeTravel"
        />
      </div>

      <div class="mt-8">
        <div class="font-bold uppercase text-sm">
          Enter Autocomplete
        </div>

        <div class="dark:text-gray-300 text-gray-800 text-sm mt-1">
          When enabled, autocomplete suggestions will be taken if the user presses the enter key.
        </div>

        <ToggleField
          class="my-2"
          title="Use Enter Autocomplete"
          v-model="settings.editor.enterAutocomplete"
        />
      </div>


      <div class="mt-8">
        <div class="font-bold uppercase text-sm">
          Dark Mode
        </div>

        <div class="dark:text-gray-300 text-gray-800 text-sm mt-1">
          When disabled, use light mode.
        </div>

        <ToggleField
          class="my-2"
          title="Use Dark Mode"
          v-model="settings.editor.darkMode"
        />
      </div>
    </div>

    
  </Modal>
</template>

<script setup lang="ts">
import { CogIcon, XMarkIcon } from '@heroicons/vue/24/solid'
import Modal from './Modal.vue'
import NumberField from './console/NumberField.vue'

import { settings } from '../state/state'
import ToggleField from './console/ToggleField.vue'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits(['close'])
</script>
