<template>
  <div
    class="text-xs font-mono flex flex-col overflow-hidden grow content-start px-2"
  >
    <div
      ref="scroll"
      class="mt-2 overflow-auto w-full h-full whitespace-pre relative"
      v-if="consoleData.console.length"
      @scroll="updateBounds"
      @mousedown.prevent="handleDown"
    >
      <div :style="{ height: `${topPadding}px` }" />

      <div
        v-for="index in renderCount"
        :key="getIndex(index)"
        class="h-4"
        :class="consoleClasses(getIndex(index))"
      >
        {{ consoleData.console[getIndex(index)] }}
      </div>

      <div :style="{ height: `${bottomPadding}px` }" />

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

      <Cursor
        :start="renderStart"
        :count="renderCount"
        :position="position"
        :cursor-shift="0"
        :line-height="lineHeight"
      />

      <SelectionOverlay
        v-if="computedRanges"
        :range="computedRanges"
        :line-height="lineHeight"
      />
    </div>

    <div v-else class="text-neutral-500">
      Nothing yet.
    </div>
  </div>
</template>

<script setup lang="ts">
import { consoleData } from '../../state/console-data'

import { useVirtualize } from '../../utils/virtualization'
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useCursor } from '../../utils/cursor'
import { Editor } from '../../utils/editor'
import { CursorState } from '../../utils/tabs'
import { settings } from '../../state/state'
import { tiny } from '../../utils/query/text-size'
import Cursor from '../Cursor.vue'
import SelectionOverlay from '../SelectionOverlay.vue'

const lineHeight = 16

const input = ref(null as HTMLInputElement | null)

const scroll = ref(null as HTMLElement | null)

const {
  renderStart,
  renderCount,
  topPadding,
  bottomPadding,
  getIndex,
  update
} = useVirtualize(lineHeight, () => consoleData.console.length)

const cursor = reactive({
  line: 0, index: 0, highlight: null
} as CursorState)

function makeEditor(): Editor {
  return new Editor(consoleData.console, cursor, () => {}, false)
}

const state = reactive({
  editor: makeEditor()
})

watch(() => consoleData.console, () => {
  state.editor = makeEditor()
})

const {
  position,
  range,
  dropCursor,
  handleKey,
  getSelection,
  pasteText,
  dropSelection,
  dragTo
} = useCursor(
  () => state.editor,
  () => cursor,
  settings.editor,
  tiny,
  16
)

const computedRanges = computed(() => {
  return range(renderStart.value, renderCount.value)
})

const mouseDown = ref(false)

function handleDown(event: MouseEvent) {
  input.value?.focus()

  const { x, y } = editorCoordinates(event)

  mouseDown.value = true
  dropCursor(x, y)
}

function editorCoordinates(event: MouseEvent): { x: number, y: number } {
  if (scroll.value) {
    const rect = scroll.value.getBoundingClientRect()

    return {
      x: event.pageX - rect.left + (scroll.value?.scrollLeft ?? 0),
      y: event.pageY - rect.top + (scroll.value?.scrollTop ?? 0)
    }
  }

  return { x: 0, y: 0 }
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

onMounted(() => {
  updateBounds()

  window.addEventListener('mousemove', handleMove)
  window.addEventListener('mouseup', handleUp)
})

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMove)
  window.removeEventListener('mouseup', handleUp)
})

const bottomCursorSpace = 24

function makeVisible(offset: number) {
  if (scroll.value) {
    if (offset < scroll.value.scrollTop) {
      scroll.value.scrollTop = offset
    } else if (offset > scroll.value.scrollTop + scroll.value.clientHeight - bottomCursorSpace) {
      scroll.value.scrollTop = offset - scroll.value.clientHeight + bottomCursorSpace
    }
  }
}

watch(() => position.value.offsetY, async (value) => {
  await nextTick()

  makeVisible(value)
})

function updateBounds() {
  if (!scroll.value) {
    return
  }

  update(scroll.value.scrollTop, scroll.value.clientHeight)
}

watch(() => consoleData.console, updateBounds)
onMounted(updateBounds)

function consoleClasses(index: number): string[] {
  const value = consoleData.consoleMeta.get(index)

  if (value) {
    return [value.highlight]
  } else {
    return []
  }
}
</script>
