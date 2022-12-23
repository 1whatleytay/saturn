<template>
  <div v-if="state.debug">
    <div class="w-full" :style="{ height: `${properties.height}px` }" />

    <div
      ref="grabber"
      class="
        w-full
        fixed
        bottom-0
        z-30
        bg-neutral-900
        border-t border-neutral-700
        flex flex-col
        max-h-screen
      " :style="{
        height: `${properties.height}px`,
        opacity: properties.height > closingHeight ? '1' : '0'
      }">
      <div
        class="absolute top-0 h-2 w-full cursor-row-resize"
        @mousedown="handleDown"
      />

      <div class="h-10 mb-2 flex items-center text-sm font-bold text-neutral-400">
        <div class="rounded-full py-0.5 px-4 text-white text-gray-300 mx-4" :class="[modeClass]">
          {{ modeString }}
        </div>

        <Tab
          title="Registers"
          :selected="properties.tab === DebugTab.Registers"
          @select="properties.tab = DebugTab.Registers"
        />

        <Tab
          title="Memory"
          :selected="properties.tab === DebugTab.Memory"
          @select="properties.tab = DebugTab.Memory"
        />

        <Tab
          title="Console"
          :selected="properties.tab === DebugTab.Console"
          @select="properties.tab = DebugTab.Console"
        />

        <button class="w-10 h-10 ml-auto
            hover:bg-slate-800
            text-slate-300
            shrink-0
            flex items-center justify-center
            font-black
          " @click="close">
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>

      <div
        v-if="properties.tab === DebugTab.Registers"
        class="font-mono text-sm overflow-scroll flex flex-col flex-wrap grow content-start"
      >
        <div
          v-for="values in registersMap" :key="values[0]"
          class="flex border-b border-neutral-700 w-52"
        >
          <div class="w-20 px-4 py-2 text-neutral-400">
            {{ values[0] }}
          </div>

          <div class="w-32 px-4 py-2 hover:bg-neutral-800 cursor-pointer">
            0x{{ values[1].toString(16) }}
          </div>
        </div>
      </div>

      <div
        v-if="properties.tab === DebugTab.Memory"
        class="text-sm overflow-scroll flex flex-col flex-wrap grow content-start"
      >
        <div class="flex items-center text-md">
          <label for="address" class="font-bold px-4 py-2">Address</label>
          <input id="address" type="text" class="bg-neutral-800 text-neutral-300 px-4 py-2 w-48" />
        </div>
      </div>

      <div
        v-if="properties.tab === DebugTab.Console"
        class="text-sm font-mono overflow-scroll flex flex-col flex-wrap grow whitespace-pre content-start"
      >
        Debug Console
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'

import { state } from '../state/editor-state'

import { XMarkIcon } from '@heroicons/vue/24/solid'
import { ExecutionMode } from '../utils/mips.js'
import Tab from './Tab.vue'

const closingHeight = 90
const defaultHeight = 320

const registers = [
  '$zero', '$at', '$v0', '$v1',
  '$a0', '$a1', '$a2', '$a3',
  '$t0', '$t1', '$t2', '$t3',
  '$t4', '$t5', '$t6', '$t7',
  '$s0', '$s1', '$s2', '$s3',
  '$s4', '$s5', '$s6', '$s7',
  '$t8', '$t9', '$k0', '$k1',
  '$gp', '$sp', '$fp', '$ra',
]

const running = computed(() => state.debug?.mode === ExecutionMode.Running)
const modeString = computed(() => {
  switch (state.debug?.mode) {
    case ExecutionMode.Running: return 'Running'
    case ExecutionMode.Breakpoint: return 'Breakpoint'
    case ExecutionMode.Paused: return 'Paused'
    case ExecutionMode.Invalid: return 'Exception'
    default: return 'Debug'
  }
})
const modeClass = computed(() => {
  switch (state.debug?.mode) {
    case ExecutionMode.Running: return 'bg-teal-900'
    case ExecutionMode.Breakpoint: return 'bg-red-900'
    case ExecutionMode.Paused: return 'bg-teal-900'
    case ExecutionMode.Invalid: return 'bg-red-900'
    default: return 'bg-teal-900'
  }
})
const registersMap = computed(() => {
  const core = registers.map((name, index) => [name, state.debug?.registers[index] ?? 0]) as [string, number][]
  const other = [['hi', state.debug?.hi ?? 0], ['lo', state.debug?.lo ?? 0]] as [string, number][]

  const result = []
  result.push(['pc', state.debug?.pc ?? 0])
  result.push(...core)
  result.push(...other)

  return result
})

enum DebugTab {
  Registers,
  Memory,
  Console
}

const properties = reactive({
  lastHeight: defaultHeight,
  height: defaultHeight,
  resizing: null as number | null,
  tab: DebugTab.Registers
})

const grabber = ref(null as HTMLElement | null)

function close() {
  state.debug = null
}

function grabberPosition(event: MouseEvent): { x: number, y: number } {
  return {
    x: event.pageX - (grabber.value?.offsetLeft ?? 0),
    y: event.pageY - (grabber.value?.offsetTop ?? 0)
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

const handleUp = (event: MouseEvent) => {
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
    state.debug = null

    properties.height = properties.lastHeight
  } else {
    properties.lastHeight = height
  }
}

watch(() => properties.height, height => {
  watchHeight(height, !!properties.resizing)
})

watch(() => properties.resizing, resizing => {
  watchHeight(properties.height, !!resizing)
})
</script>
