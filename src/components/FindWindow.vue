<template>
  <div v-if="find.state.show" class="py-2 h-12 relative w-full block shrink-0">
    <div
      class="flex items-center h-12 border-b border-neutral-700 bg-neutral-900 w-full absolute top-0 z-30"
    >
      <label for="find" class="text-xs font-bold px-4 py-2">Find</label>
      <input
        id="find"
        ref="findInput"
        type="text"
        autocomplete="off"
        spellcheck="false"
        @keydown.esc.prevent="close()"
        class="text-xs font-mono bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
        v-model="find.state.text"
      />

      <div class="flex px-2 space-x-1">
        <button class="p-1 rounded hover:bg-neutral-700">
          <ArrowUpIcon class="w-4 h-4" />
        </button>

        <button class="p-1 rounded hover:bg-neutral-700">
          <ArrowDownIcon class="w-4 h-4" />
        </button>
      </div>

      <div class="text-neutral-600 text-sm">
        {{ count }} matches
      </div>

      <button class="
        w-12 h-12 ml-auto
        hover:bg-slate-800
        text-slate-300
        shrink-0
        flex items-center justify-center
      " @click="close()">
        <XMarkIcon class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowUpIcon, ArrowDownIcon, XMarkIcon } from '@heroicons/vue/24/solid'
import { find } from '../state/state'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

const count = computed(() => {
  return find.state.matches
    .map(x => x.length)
    .reduce((a, b) => a + b, 0)
})

const findInput = ref(null as HTMLElement | null)

let needsFocus = false

function close() {
  find.state.show = false
}

function queueFocus() {
  if (findInput.value) {
    findInput.value.focus()
    needsFocus = false
  } else {
    needsFocus = true
  }
}

const listenEscape = (event: KeyboardEvent) => {
  if (event.key == 'Escape') {
    find.state.show = false
  }
}

onMounted(() => {
  window.addEventListener('keydown', listenEscape)
})

onUnmounted(() => {
  window.removeEventListener('keydown', listenEscape)
})

watch(() => find.state.show, (value, old) => {
  if (value && !old) {
    queueFocus()
  }
})

watch(() => find.state.focus, value => {
  if (value) {
    queueFocus()
    find.state.focus = false
  }
})

watch(() => findInput.value, () => {
  if (needsFocus) {
    queueFocus()
  }
})
</script>
