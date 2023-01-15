<template>
  <div
    ref="scroll"
    class="font-mono text-sm flex-auto flex-grow overflow-auto flex pt-2"
    @scroll="handleScroll"
  >
    <div
      class="w-16 pr-2 mr-2 text-xs text-slate-600 shrink-0 z-10 fixed left-0 bg-neutral-900 pt-2"
      @wheel.stop
      :style="{ top: `${linesOffset}px` }"
    >
      <div
        v-for="(_, index) in tabBody" :key="index"
        @click="toggleBreakpoint(index)"
        class="w-full h-6 text-right flex items-center justify-end cursor-pointer pointer-events-auto"
      >
        <div
          v-if="hasBreakpoint(index)"
          class="rounded-full bg-red-700 w-3 h-3 mr-auto ml-3"
        />

        {{ index + 1 }}
      </div>
    </div>

    <div class="w-16 pr-2 mr-2 shrink-0">
    </div>

    <div
      ref="code"
      class="flex-grow cursor-text text-sm relative outline-none whitespace-pre"
      @mousedown.prevent="handleDown"
    >
      <input
        type="text"
        ref="input"
        spellcheck="false"
        :value="''"
        tabindex="0"
        class="opacity-0 pointer-events-none fixed top-0 left-0 peer"
        @keydown="handleKey"
        @copy.prevent="handleCopy"
        @cut.prevent="handleCut"
        @paste.prevent="handlePaste"
      />

      <div
        v-for="(line, index) in tabBody"
        :key="index"
        class="h-6 flex items-center pr-16"
        :class="{
          'bg-breakpoint-neutral': hasBreakpoint(index) && index !== stoppedIndex,
          'bg-breakpoint-stopped': index === stoppedIndex
        }"
      >
        {{ line }}
      </div>

      <div class="h-32" />

      <Cursor />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { tab } from '../state/tabs-state'
import { consoleData } from '../state/console-data'
import {
  cursor,
  tabBody,
  putCursor,
  handleKey,
  dropCursor,
  dragTo,
  pasteText,
  getSelection,
  clearSelection,
  dropSelection,
  lineStart
} from '../state/cursor-state'
import { setBreakpoint } from '../utils/editor-debug'
import Cursor from './Cursor.vue'

const linesOffset = ref(0)

const scroll = ref(null as HTMLElement | null)
const code = ref(null as HTMLElement | null)
const input = ref(null as HTMLElement | null)

const stoppedIndex = computed(() => {
  const profile = tab()?.profile
  const registers = consoleData.registers
  const execution = consoleData.execution

  if (!profile || !registers || !execution) {
    return null
  }

  // Reactivity concern here (eh... not too bad, we just want to listen to changes in debug).
  const point = execution.breakpoints?.pcToLine.get(registers.pc)

  return point ?? null
})

function handleScroll() {
  if (!scroll.value) {
    return
  }

  const point = scroll.value.scrollTop - scroll.value.offsetTop

  linesOffset.value = -point
}

const focusHandler = () => {
  input?.value?.focus()
}

function editorCoordinates(event: MouseEvent): { x: number, y: number } {
  if (code.value) {
    return {
      x: event.pageX - code.value.offsetLeft + (scroll.value?.scrollLeft ?? 0),
      y: event.pageY - code.value.offsetTop + (scroll.value?.scrollTop ?? 0)
    }
  }

  return { x: 0, y: 0 }
}

function hasBreakpoint(index: number): boolean {
  return tab()?.breakpoints.includes(index) ?? false
}

async function toggleBreakpoint(index: number) {
  const state = tab()

  if (!state) {
    return
  }

  const remove = state.breakpoints.includes(index)

  await setBreakpoint(index, remove)
}

const mouseDown = ref(false)

function handleDown(event: MouseEvent) {
  focusHandler()

  const { x, y } = editorCoordinates(event)

  mouseDown.value = true
  dropCursor(x, y)
}

const handleMove = (event: MouseEvent) => {
  if ((event.buttons & 1) > 0 && mouseDown.value) {
    const { x, y } = editorCoordinates(event)
    dragTo(x, y)
  } else {
    mouseDown.value = false
  }
}

const handleUp = () => {
  mouseDown.value = false
}

function handleCopy(event: ClipboardEvent) {
  if (!event.clipboardData) {
    return
  }

  const selection = getSelection()

  if (selection) {
    event.clipboardData.setData('text/plain', selection)
  }
}

function handleCut(event: ClipboardEvent) {
  if (!event.clipboardData) {
    return
  }

  const selection = getSelection()
  dropSelection()

  if (selection) {
    event.clipboardData.setData('text/plain', selection)
  }
}

function handlePaste(event: ClipboardEvent) {
  if (event.clipboardData) {
    const value = event.clipboardData.getData('text/plain')

    pasteText(value)
  }
}

onMounted(() => {
  putCursor({ line: 0, index: 0 })
  handleScroll()

  window.addEventListener('focus', focusHandler)
  window.addEventListener('mousemove', handleMove)
  window.addEventListener('mouseup', handleUp)
})

onUnmounted(() => {
  window.removeEventListener('focus', focusHandler)
  window.removeEventListener('mousemove', handleMove)
  window.removeEventListener('mouseup', handleUp)
})

const bottomCursorSpace = 42

function makeVisible(offset: number) {
  if (scroll.value) {
    if (offset < scroll.value.scrollTop) {
      scroll.value.scrollTop = offset
    } else if (offset > scroll.value.scrollTop + scroll.value.clientHeight - bottomCursorSpace) {
      scroll.value.scrollTop = offset - scroll.value.clientHeight + bottomCursorSpace
    }
  }
}

watch(() => cursor.offsetY, (value) => {
  makeVisible(value)
})

watch(() => tab()?.uuid, () => {
  clearSelection()
  putCursor({ line: 0, index: 0 })
})

watch(() => stoppedIndex.value, (index) => {
  if (index) {
    const start = lineStart(index)

    makeVisible(start)
  }
})
</script>
