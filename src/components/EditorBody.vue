<template>
  <div
    ref="scroll"
    class="font-mono text-sm flex-auto flex-grow overflow-scroll flex pt-2"
    @scroll="handleScroll"
  >
    <div
      class="w-16 pr-2 mr-2 text-xs text-slate-600 shrink-0 z-10 fixed left-0 bg-neutral-900 pt-2"
      @wheel.stop
      :style="{ top: `${linesOffset}px` }"
    >
      <div
        v-for="(_, index) in lines" :key="index"
        @click="setBreakpoint(index)"
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
        v-for="(line, index) in lines"
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

      <div class="absolute top-0 pointer-events-none" v-if="cursor.highlight">
        <div
          v-for="(line, index) in lines"
          :key="index"
          class="h-6 flex items-center"
        >
          <div v-if="hasSelection(index)">
            <span class="opacity-0">
              {{ leadingSelection(line, index) }}
            </span>

            <span class="rounded opacity-30 px-1 -mx-1 bg-blue-500">
              <span class="opacity-0">
                {{ bodySelection(line, index) }}
              </span>
            </span>
          </div>
        </div>
      </div>

      <div
        class="w-0.5 h-6 bg-orange-400 hidden peer-focus:block absolute mx-[-0.08rem]"
        :style="{
          left: `${cursor.offsetX}px`,
          top: `${cursor.offsetY}px`
        }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { state, tab } from '../state/editor-state'
import {
  lines,
  cursor,
  putCursor,
  handleKey,
  dropCursor,
  dragTo,
  paste,
  getSelection,
  clearSelection,
  dropSelection, lineStart
} from '../state/editor-cursor'

const linesOffset = ref(0)

const scroll = ref(null as HTMLElement | null)
const code = ref(null as HTMLElement | null)
const input = ref(null as HTMLElement | null)

const stoppedIndex = computed(() => {
  const profile = tab()?.profile
  const debug = state.debug

  if (!profile || !debug || !state.execution) {
    return null
  }

  const point = Object.entries(profile.breakpoints)
    .find(([key, value]) => value === debug.pc)

  if (!point) {
    return null
  }

  try {
    return parseInt(point[0])
  } catch {
    return null
  }
})

function handleScroll() {
  if (!scroll.value) {
    return
  }

  const point = scroll.value.scrollTop - scroll.value.offsetTop

  linesOffset.value = -point
}

function hasSelection(index: number): boolean {
  // assert cursor.highlight !== null

  const min = Math.min(cursor.highlight!.line, cursor.line)
  const max = Math.max(cursor.highlight!.line, cursor.line)

  return min <= index && index <= max
}

function lineIndices(index: number): number[] {
  // assert cursor.highlight !== null

  return [
    ...(cursor.highlight!.line == index ? [cursor.highlight!.index] : []),
    ...(cursor.line == index ? [cursor.index] : []),
  ]
}

function leadingSelection(line: string, index: number) {
  // assert cursor.highlight !== null

  const indices = lineIndices(index)
  if (!indices.length) {
    return ''
  }

  const min = Math.min(...indices)

  if (indices.length === 2 || (cursor.highlight!.line === index) != (cursor.highlight!.line > cursor.line)) {
    return line.substring(0, min)
  } else {
    return ''
  }
}

function bodySelection(line: string, index: number) {
  // assert cursor.highlight !== null

  const indices = lineIndices(index)
  if (!indices.length) {
    return line
  }

  if (indices.length === 2) {
    const min = Math.min(...indices)
    const max = Math.max(...indices)

    return line.substring(min, max)
  }

  if (cursor.highlight!.line > cursor.line) {
    if (cursor.highlight!.line === index) {
      return line.substring(0, cursor.highlight!.index)
    } else {
      return line.substring(cursor.index)
    }
  } else {
    if (cursor.highlight!.line === index) {
      return line.substring(cursor.highlight!.index)
    } else {
      return line.substring(0, cursor.index)
    }
  }
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

function setBreakpoint(index: number) {
  const state = tab()

  if (!state) {
    return
  }

  if (state.breakpoints.includes(index)) {
    state.breakpoints = state.breakpoints.filter(point => point !== index)
  } else {
    state.breakpoints.push(index)
  }
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

    paste(value)
  }
}

onMounted(() => {
  putCursor(0, 0)
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
  putCursor(0, 0)
})

watch(() => stoppedIndex.value, (index) => {
  if (index) {
    const start = lineStart(index)

    makeVisible(start)
  }
})
</script>
