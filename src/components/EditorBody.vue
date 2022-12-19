<template>
  <div ref="scroll" class="font-mono text-sm flex-auto flex-grow overflow-scroll flex mt-2">
    <div class="w-10 mr-5 text-xs text-slate-600">
      <div
        v-for="(_, index) in lines" :key="index"
        class="w-full h-6 text-right flex items-center justify-end"
      >
        {{ index + 1 }}
      </div>
    </div>

    <div
      ref="code"
      class="flex-grow cursor-text text-sm relative outline-none whitespace-pre group"
      @mousedown.prevent="handleDown"
    >
      <input
        type="text"
        ref="input"
        spellcheck="false"
        :value="''"
        tabindex="0"
        class="opacity-0 pointer-events-none absolute top-0 left-0 peer"
        @keydown="handleKey"
      />

      <div v-for="(line, index) in lines" :key="index" class="h-6 flex items-center">
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
import { onMounted, onUnmounted, reactive, ref, watch } from 'vue'

import { tab } from '../state/editor-state'
import {
  lines,
  cursor,
  putCursor,
  handleKey,
  dropCursor,
  dragTo,
  clearSelection
} from '../state/editor-cursor'

const scroll = ref(null as HTMLElement | null)
const code = ref(null as HTMLElement | null)
const input = ref(null as HTMLElement | null)

interface SelectionPartsResult {
  leading: string,
  selected: string,
  trailing: string
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
  console.log(input)
  input?.value?.focus()
}

function editorCoordinates(event: MouseEvent): { x: number, y: number } {
  if (code.value) {
    return {
      x: event.pageX - code.value.offsetLeft,
      y: event.pageY - code.value.offsetTop + (scroll?.value?.scrollTop ?? 0)
    }
  }

  return { x: 0, y: 0 }
}

const mouseDown = ref(false)

function handleDown(event: MouseEvent) {
  focusHandler()

  const { x, y } = editorCoordinates(event)
  mouseDown.value = true

  dropCursor(x, y)
}

const handleMove = (event: MouseEvent) => {
  if (mouseDown.value) {
    const { x, y } = editorCoordinates(event)
    dragTo(x, y)
  }
}

const handleUp = (event: MouseEvent) => {
  mouseDown.value = false
}

function handlePaste(event: Event) {
  console.log(event)
  alert('bro')
}

onMounted(() => {
  putCursor(0, 0)

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

watch(() => cursor.offsetY, (value) => {
  if (scroll.value) {
    if (value < scroll.value.scrollTop) {
      scroll.value.scrollTop = value
    } else if (value > scroll.value.scrollTop + scroll.value.clientHeight - bottomCursorSpace) {
      scroll.value.scrollTop = value - scroll.value.clientHeight + bottomCursorSpace
    }
  }
})

watch(() => tab()?.uuid, () => {
  clearSelection()
  putCursor(0, 0)
})
</script>
