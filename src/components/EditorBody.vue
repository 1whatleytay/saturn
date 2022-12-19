<template>
  <div class="font-mono text-sm flex-auto flex-grow overflow-scroll flex mt-2">
    <div class="w-10 mr-5 text-xs text-slate-600">
      <div
        v-for="(_, index) in lines"
        class="w-full h-6 text-right flex items-center justify-end"
      >
        {{ index + 1 }}
      </div>
    </div>

    <div tabindex="0" ref="handler" class="flex-grow cursor-text text-sm relative outline-none" @click="focusHandler()" @keydown="handleKey">
      <div v-for="line in lines" class="h-6 flex items-center whitespace-pre">
        {{ line }}
      </div>

      <div
        class="w-0.5 h-6 bg-orange-400 absolute mx-[-0.08rem]"
        :style="{
        left: `${cursor.offsetX}px`,
        top: `${cursor.offsetY}px`
      }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'

import { tab } from '../state/editor-state'
import { lines, cursor, putCursor, handleKey } from '../state/editor-cursor'

const handler = ref(null as HTMLElement | null)

function focusHandler() {
  handler?.value?.focus()
}

const onFocus = () => {
  focusHandler()
}

onMounted(() => {
  putCursor(0, 0)

  window.addEventListener('focus', onFocus)
})

onUnmounted(() => {
  window.removeEventListener('focus', onFocus)
})

watch(() => tab()?.uuid, () => {
  putCursor(0, 0)
})
</script>
