<template>
  <Modal :show="props.show" @close="emit('close')">
    <div
      class="pointer-events-auto mx-auto flex max-h-[84vh] max-w-2xl flex-col overflow-y-scroll rounded-xl bg-white px-8 py-6 shadow dark:bg-neutral-900"
    >
      <div
        class="my-2 flex w-full shrink-0 items-center text-2xl font-semibold"
      >
        <CogIcon class="mr-3 h-7 w-7 shrink-0" /> Settings

        <button
          class="ml-auto flex h-8 w-8 shrink-0 items-center justify-center rounded text-slate-800 hover:bg-slate-300 dark:text-slate-300 dark:hover:bg-slate-800"
          @click="emit('close')"
        >
          <XMarkIcon class="h-4 w-4" />
        </button>
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">Font Size</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          Adjust the size of the text in the editor window.
        </div>

        <NumberField class="my-2 text-sm" v-model="settings.editor.fontSize" />
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">Tab Size</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
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
        <div class="text-sm font-bold uppercase">Time Travel</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          When enabled, allows fpr stepping backwards when running your program.
          Enabling may affect performance.
        </div>

        <ToggleField
          class="my-2"
          title="Use Time Travel"
          v-model="settings.execution.timeTravel"
        />
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">Enter Autocomplete</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          When enabled, autocomplete suggestions will be taken if the user
          presses the enter key.
        </div>

        <ToggleField
          class="my-2"
          title="Use Enter Autocomplete"
          v-model="settings.editor.enterAutocomplete"
        />
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">Dark Mode</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          When disabled, use light mode.
        </div>

        <ToggleField
          class="my-2"
          title="Use Dark Mode"
          v-model="settings.editor.darkMode"
        />
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">Show minimap</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          When enabled, a minimap will be shown on the right side of the editor.
        </div>

        <ToggleField
          class="my-2"
          title="Show minimap"
          v-model="settings.editor.showMinimap"
        />
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">VIM Mode</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          When enabled, use VIM keybindings.
        </div>

        <ToggleField
          class="my-2"
          title="Use VIM Mode"
          v-model="settings.editor.vimMode"
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
