<template>
  <div v-if="find.state.show" class="py-2 h-12 relative w-full block shrink-0">
    <div
      class="flex items-center h-12 border-b border-neutral-700 dark:bg-neutral-900 bg-neutral-200 w-full absolute top-0 z-30"
    >
      <label for="find" class="text-xs font-bold px-4 py-2">Find</label>
      <input
        id="find"
        ref="findInput"
        type="text"
        autocomplete="off"
        spellcheck="false"
        @keydown.enter="jumpToNext()"
        @keydown.esc.prevent="close()"
        class="text-xs font-mono dark:bg-neutral-800 bg-neutral-300 dark:text-neutral-300 text-neutral-700 px-2 py-1 w-64 rounded"
        v-model="find.state.text"
      />

      <div class="flex px-2 space-x-1">
        <button class="p-1 rounded dark:hover:bg-neutral-700 hover:bg-neutral-300" @click="jumpToNext()">
          <ArrowRightIcon class="w-4 h-4" />
        </button>
      </div>

      <div class="text-neutral-600 text-sm">{{ count }} matches</div>

      <button
        class="w-12 h-12 ml-auto dark:hover:bg-slate-800 hover:bg-slate-300 dark:text-slate-300 text-slate-800 shrink-0 flex items-center justify-center"
        @click="close()"
      >
        <XMarkIcon class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowRightIcon, XMarkIcon } from '@heroicons/vue/24/solid'
import { find, jump, tab } from '../state/state'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

function jumpToNext() {
  const current = tab()?.cursor

  if (!current) {
    return
  }

  const next = find.nextItem(current.line, current.index)

  if (!next) {
    return
  }

  jump({ line: next.line, index: next.match.index })
}

const count = computed(() => {
  return find.state.matches.map((x) => x.length).reduce((a, b) => a + b, 0)
})

const findInput = ref(null as HTMLInputElement | null)

let needsFocus = false

function close() {
  find.state.show = false
}

function queueFocus() {
  if (findInput.value) {
    findInput.value.focus()
    findInput.value.select()
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

watch(
  () => find.state.show,
  (value, old) => {
    if (value && !old) {
      queueFocus()
    }
  }
)

watch(
  () => find.state.focus,
  (value) => {
    if (value) {
      queueFocus()
      find.state.focus = false
    }
  }
)

watch(
  () => findInput.value,
  () => {
    if (needsFocus) {
      queueFocus()
    }
  }
)
</script>
