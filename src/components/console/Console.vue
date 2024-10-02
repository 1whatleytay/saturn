<template>
  <div v-if="consoleData.showConsole">
    <div
      v-if="properties.height > closingHeight"
      class="w-full"
      :style="{ height: `${properties.height}px` }"
    />

    <div
      ref="grabber"
      class="w-full fixed bottom-0 z-30 dark:bg-neutral-900 bg-neutral-200 border-t border-neutral-700 flex flex-col max-h-screen"
      :style="{
        height: `${properties.height}px`,
        opacity: properties.height > closingHeight ? '1' : '0',
      }"
    >
      <div
        class="absolute top-0 -mt-1 h-3 w-full cursor-row-resize"
        @mousedown="handleDown"
      />

      <div class="h-10 flex items-center text-sm font-bold text-neutral-400">
        <div
          class="rounded-full py-0.5 px-4 mx-4"
          :class="[modeClass]"
        >
          {{ modeString }}
        </div>

        <Tab
          title="Debug"
          :selected="consoleData.tab === DebugTab.Debug"
          @mousedown="() => (consoleData.tab = DebugTab.Debug)"
        />

        <Tab
          title="Console"
          :selected="consoleData.tab === DebugTab.Console"
          @mousedown="() => (consoleData.tab = DebugTab.Console)"
        />

        <Tab
          title="Registers"
          :selected="consoleData.tab === DebugTab.Registers"
          @mousedown="() => (consoleData.tab = DebugTab.Registers)"
        />

        <Tab
          title="Memory"
          :selected="consoleData.tab === DebugTab.Memory"
          @mousedown="() => (consoleData.tab = DebugTab.Memory)"
        />

        <Tab
          title="Bitmap"
          :selected="consoleData.tab === DebugTab.Bitmap"
          @mousedown="() => (consoleData.tab = DebugTab.Bitmap)"
        />

        <button
          class="w-10 h-10 ml-auto dark:hover:bg-slate-800 hover:bg-slate-300 dark:text-slate-300 text-slate-800 shrink-0 flex items-center justify-center"
          @click="close"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>

      <RegistersTab v-if="consoleData.tab === DebugTab.Registers" />
      <MemoryTab v-if="consoleData.tab === DebugTab.Memory" />
      <ConsoleTab v-if="consoleData.tab === DebugTab.Console" />
      <BitmapTab v-if="consoleData.tab === DebugTab.Bitmap" />
      <TestsTab v-if="consoleData.tab === DebugTab.Tests" />
      <BreakTab v-if="consoleData.tab === DebugTab.Debug" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'

import { consoleData, DebugTab } from '../../state/console-data'

import { XMarkIcon } from '@heroicons/vue/24/solid'
import { ExecutionModeType } from '../../utils/mips/mips'
import Tab from '../Tab.vue'
import RegistersTab from './RegistersTab.vue'
import MemoryTab from './MemoryTab.vue'
import ConsoleTab from './ConsoleTab.vue'
import BitmapTab from './BitmapTab.vue'
import TestsTab from './TestsTab.vue'
import BreakTab from './BreakTab.vue'

const closingHeight = 90
const defaultHeight = 320

const modeString = computed(() => {
  switch (consoleData.mode) {
    case ExecutionModeType.Running:
      return 'Running'
    case ExecutionModeType.Breakpoint:
      return 'Breakpoint'
    case ExecutionModeType.Paused:
      return 'Paused'
    case ExecutionModeType.Invalid:
      return 'Exception'
    case ExecutionModeType.Finished:
      return 'Finished'
    default:
      return 'Debug'
  }
})
const modeClass = computed(() => {
  switch (consoleData.mode) {
    case ExecutionModeType.Running:
      return 'dark:text-teal-100 text-teal-800 border-teal-500 border'
    case ExecutionModeType.Breakpoint:
    case ExecutionModeType.Invalid:
      return 'dark:text-red-100 text-red-800 border-red-400 border'
    case ExecutionModeType.Paused:
      return 'dark:text-yellow-100 text-yellow-800 border-yellow-400 border'
    case ExecutionModeType.Finished:
      return 'dark:text-lime-100 text-lime-800 border-lime-500 border'
    default:
      return 'dark:text-white text-black'
  }
})

const properties = reactive({
  lastHeight: defaultHeight,
  height: defaultHeight,
  resizing: null as number | null,
})

const grabber = ref(null as HTMLElement | null)

function close() {
  consoleData.showConsole = false
}

function grabberPosition(event: MouseEvent): { x: number; y: number } {
  return {
    x: event.pageX - (grabber.value?.offsetLeft ?? 0),
    y: event.pageY - (grabber.value?.offsetTop ?? 0),
  }
}

function handleDown(event: MouseEvent) {
  const position = grabberPosition(event)

  if (!position) {
    return
  }

  properties.resizing = position.y
}

const handleMove = (event: MouseEvent) => {
  // properties.resizing = posY
  if (!properties.resizing) {
    return
  }

  if ((event.buttons & 1) === 0) {
    properties.resizing = null

    return
  }

  const position = grabberPosition(event)

  if (!position) {
    return
  }

  const difference = properties.resizing - position.y

  properties.height += difference
}

const handleUp = () => {
  properties.resizing = null
}

onMounted(() => {
  window.addEventListener('mousemove', handleMove)
  window.addEventListener('mouseup', handleUp)
})

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMove)
  window.removeEventListener('mouseup', handleUp)
})

function watchHeight(height: number, resizing: boolean) {
  if (resizing) {
    return
  }

  if (height < closingHeight) {
    consoleData.showConsole = false

    properties.height = properties.lastHeight
  } else {
    properties.lastHeight = height
  }
}

watch(
  () => properties.height,
  (height) => {
    watchHeight(height, !!properties.resizing)
  }
)

watch(
  () => properties.resizing,
  (resizing) => {
    watchHeight(properties.height, !!resizing)
  }
)
</script>
