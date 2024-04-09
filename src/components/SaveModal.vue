<template>
  <Modal :show="!!props.dialog.state.tab">
    <div
      class="max-w-lg dark:bg-neutral-900 bg-neutral-200 rounded-xl px-8 py-6 mx-auto flex flex-col shadow"
    >
      <div>Would you like to save changes made to this file?</div>

      <div class="text-neutral-400 text-xs font-medium mt-1">
        You will lose your changes if you chose to not save them.
      </div>

      <div class="mt-4 text-sm flex flex-wrap">
        <button
          @click="props.dialog.selectSave()"
          ref="saveButton"
          id="save-modal-save"
          class="mr-4 mt-4 w-24 dark:bg-slate-800 dark:hover:bg-slate-700 bg-slate-400 hover:bg-slate-500 transition-colors duration-150 px-4 py-2 rounded"
        >
          Save
        </button>

        <button
          @click="props.dialog.selectDiscard()"
          ref="discardButton"
          id="save-modal-discard"
          class="mr-8 mt-4 w-28 dark:bg-slate-800 dark:hover:bg-slate-700 bg-slate-400 hover:bg-slate-500 transition-colors duration-150 px-4 py-2 rounded"
        >
          Don't Save
        </button>

        <button
          @click="props.dialog.selectDismiss()"
          ref="dismissButton"
          id="save-modal-dismiss"
          class="mt-4 w-24 dark:bg-slate-800 dark:hover:bg-slate-700 bg-slate-400 hover:bg-slate-500 transition-colors duration-150 px-4 py-2 rounded"
        >
          Cancel
        </button>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { SaveModalResult } from '../utils/save-modal'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import Modal from './Modal.vue'

const saveButton = ref(null as HTMLButtonElement | null)
const discardButton = ref(null as HTMLButtonElement | null)
const dismissButton = ref(null as HTMLButtonElement | null)

const props = defineProps<{
  dialog: SaveModalResult
}>()

function leftButton(id: string | null): HTMLButtonElement | null {
  switch (id) {
    case 'save-modal-save':
      return dismissButton.value
    case 'save-modal-discard':
      return saveButton.value
    case 'save-modal-dismiss':
      return discardButton.value
    default:
      return null
  }
}

function rightButton(id: string | null): HTMLButtonElement | null {
  switch (id) {
    case 'save-modal-save':
      return discardButton.value
    case 'save-modal-discard':
      return dismissButton.value
    case 'save-modal-dismiss':
      return saveButton.value
    default:
      return null
  }
}

const listener = (event: KeyboardEvent) => {
  switch (event.key) {
    case 'ArrowLeft': {
      const left = leftButton(document.activeElement?.id ?? null)

      if (left) {
        left.focus()
      }

      break
    }

    case 'ArrowRight': {
      const right = rightButton(document.activeElement?.id ?? null)

      if (right) {
        right.focus()
      }

      break
    }

    case 'Escape':
      props.dialog.selectDismiss()
      break

    default:
      break
  }
}

onMounted(() => {
  window.addEventListener('keydown', listener)
})

onUnmounted(() => {
  window.removeEventListener('keydown', listener)
})

watch(
  () => props.dialog.state.tab,
  async (value, old) => {
    if (value && !old) {
      await nextTick()

      saveButton.value?.focus()
    }
  }
)
</script>
