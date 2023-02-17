<template>
  <div
    ref="scroll"
    class="font-mono text-sm flex-auto flex-grow overflow-auto flex pt-2"
    @scroll="handleScroll"
    @resize="updateBounds"
  >
    <div
      class="w-16 pr-2 mr-2 text-xs text-slate-600 shrink-0 z-10 absolute left-0 bg-neutral-900 pt-2"
      @click.self
      @wheel.stop
      :style="{ top: `${lineOffset}px` }"
    >
      <div :style="{ height: `${topPadding}px` }" />

      <div
        v-for="i in renderCount" :key="getIndex(i)"
        @click="toggleBreakpoint(getIndex(i))"
        class="w-full h-6 text-right flex items-center justify-end cursor-pointer pointer-events-auto"
      >
        <div
          v-if="hasBreakpoint(getIndex(i))"
          class="rounded-full bg-red-700 w-3 h-3 mr-auto ml-3"
        />

        {{ getIndex(i) + 1 }}
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
        autocomplete="off"
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

      <div :style="{ height: `${topPadding}px` }" />

      <div
        v-for="i in renderCount"
        :key="getIndex(i)"
        class="h-6 flex items-center pr-16"
        :class="lineStyling(i)"
      >
        <div>
          <span
            v-for="(token, index) in storage.highlights[getIndex(i)]"
            :key="index"
            :class="[token.color]"
          >
            {{ token.text }}
          </span>
        </div>
      </div>

      <div :style="{ height: `${bottomPadding}px` }" />
      <div class="h-32" />

      <Cursor :position="position" :start="renderStart" :count="renderCount" />
      <SelectionOverlay v-if="computedRanges" :range="computedRanges" />
      <Suggestions />
      <FindOverlay :start="renderStart" :count="renderCount" />
      <ErrorOverlay />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, StyleValue, watch } from 'vue'

import { consoleData } from '../state/console-data'
import { setBreakpoint } from '../utils/debug'
import { useVirtualize } from '../utils/virtualization'
import {
  range,
  getSelection,
  dropSelection,
  lineStart,
  position,
  dropCursor,
  dragTo,
  pasteText,
  storage,
  handleKey,
  find,
  tab,
  tabBody
} from '../state/state'

import Cursor from './Cursor.vue'
import FindOverlay from './FindOverlay.vue'
import ErrorOverlay from './ErrorOverlay.vue'
import Suggestions from './Suggestions.vue'
import SelectionOverlay from './SelectionOverlay.vue'

const lineOffset = ref(0)

const lineHeight = 24 // h-6 -> 1.5rem -> 24px

const {
  getIndex,
  renderStart,
  renderCount,
  topPadding,
  bottomPadding,
  update
} = useVirtualize(lineHeight, () => tabBody.value.length)

const computedRanges = computed(() => {
  return range(renderStart.value, renderCount.value)
})

function lineStyling(i: number): Record<string, boolean> {
  const index = getIndex(i)
  const breakpoint = hasBreakpoint(index)
  const isStopped = index === stoppedIndex.value

  return {
    'bg-breakpoint-neutral': breakpoint && !isStopped,
    'bg-breakpoint-stopped': isStopped,
    'bg-yellow-500 bg-opacity-5': !breakpoint && !isStopped && !(tab()?.writable ?? true)
  }
}

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

function updateBounds() {
  if (!scroll.value) {
    return
  }

  update(scroll.value.scrollTop, scroll.value.clientHeight)
}

watch(() => tabBody.value, () => { updateBounds() })

function handleScroll() {
  if (!scroll.value) {
    return
  }

  updateBounds()
  lineOffset.value = scroll.value.offsetTop - scroll.value.scrollTop
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
  input.value?.focus()

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
  handleScroll()

  window.addEventListener('mousemove', handleMove)
  window.addEventListener('mouseup', handleUp)
})

onUnmounted(() => {
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

const updateAndShow = async (value: number) => {
  await nextTick()

  makeVisible(value)
}

watch(() => position.value.offsetX, () => updateAndShow(position.value.offsetY))
watch(() => position.value.offsetY, updateAndShow)

watch(() => stoppedIndex.value, (index) => {
  if (index) {
    const start = lineStart(index)

    makeVisible(start)
  }
})

watch(() => find.state.show, async () => {
  // Wait until resize occurs...
  await nextTick()

  handleScroll()
})
</script>
