<template>
  <div v-if="state.debug">
    <div
      v-if="properties.height > closingHeight"
      class="w-full"
      :style="{ height: `${properties.height}px` }"
    />

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

      <div class="h-10 flex items-center text-sm font-bold text-neutral-400">
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
        class="text-sm overflow-scroll flex flex-col grow content-start relative"
      >
        <div class="flex items-center h-12 border-b border-neutral-700 fixed bg-neutral-900 w-full">
          <label for="address" class="text-xs font-bold px-4 py-2">Address</label>
          <input
            id="address"
            type="text"
            class="text-xs font-mono bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
            v-model="memory.address"
          />

          <div class="flex px-2 space-x-1">
            <button class="p-1 rounded hover:bg-neutral-700" @click="moveAddress(+1)">
              <ArrowUpIcon class="w-4 h-4" />
            </button>

            <button class="p-1 rounded hover:bg-neutral-700" @click="moveAddress(-1)">
              <ArrowDownIcon class="w-4 h-4" />
            </button>
          </div>

          <label for="data-type" class="text-xs font-bold px-4 py-2">Type</label>
          <select
            id="data-type"
            class="appearance-none text-xs bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
            :value="memory.mode"
            @input="setMode"
          >
            <option :value="AddressingMode.Byte">Byte</option>
            <option :value="AddressingMode.Half">Half</option>
            <option :value="AddressingMode.Word">Word</option>
          </select>
        </div>

        <!-- Height is ignored? -->
        <div class="mb-16" />

        <div class="text-right">
          <div class="flex font-bold text-neutral-500">
            <div class="w-32 px-2 py-1 shrink-0">Address</div>

            <div
              v-for="(item, index) in table?.header"
              :key="index"
              class="w-28 flex items-center px-2 py-1 shrink-0"
            >
              {{ item }}
            </div>
          </div>

          <div v-for="(row, index) in table?.rows" :key="index" class="flex font-mono">
            <div class="w-32 px-2 py-1 text-neutral-500 shrink-0">{{ row.header }}</div>

            <div
              v-for="(item, index) in row.items"
              :key="index"
              class="hover:bg-neutral-700 w-28 flex items-center px-2 py-1 shrink-0"
            >
              {{ item }}
            </div>
          </div>
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

import { ArrowDownIcon, ArrowUpIcon, XMarkIcon } from '@heroicons/vue/24/solid'
import { ExecutionMode } from '../utils/mips.js'
import Tab from './Tab.vue'

const closingHeight = 90
const defaultHeight = 320

enum AddressingMode {
  Byte,
  Half,
  Word,
}

function addressingModeSize(mode: AddressingMode) {
  switch (mode) {
    case AddressingMode.Byte: return 1
    case AddressingMode.Half: return 2
    case AddressingMode.Word: return 4
  }
}

function setMode(event: Event) {
  const input = event.target as HTMLInputElement

  try {
    memory.mode = parseInt(input.value) as AddressingMode
  } catch { }
}

function shift(bytes: number[]): string {
  let result = ''

  for (const byte of bytes) {
    result = byte.toString(16).padStart(2, '0') + result
  }

  return result
}

function parseAddress(address: string): number | null {
  if (address.startsWith('0x')) {
    address = address.substring(2)
  }

  try {
    return parseInt(address, 16)
  } catch {
    return null
  }
}

function addressString(value: number): string {
  return `0x${value.toString(16).padStart(8, '0')}`
}

const targetRows = 32
const targetColumns = 8
const defaultAddress = 0x10010000

const memory = reactive({
  address: addressString(defaultAddress),
  data: Array(256).fill(0),
  mode: AddressingMode.Word
})

function pageSize() {
  const unitSize = addressingModeSize(memory.mode)

  return targetRows * targetColumns * unitSize
}

function moveAddress(direction: number) {
  const value = parseAddress(memory.address) ?? defaultAddress
  const section = pageSize()

  memory.address = addressString(value + section * direction)
}

interface MemoryRow {
  header: string,
  items: string[]
}

interface MemoryTable {
  header: string[]
  rows: MemoryRow[]
}

const table = computed((): MemoryTable | null => {
  const address = parseAddress(memory.address)

  const unitSize = addressingModeSize(memory.mode)

  const data = memory.data
  let index = 0

  const result = [] as MemoryRow[]

  for (let row = 0; row < targetRows; row++) {
    if (!address) {
      result.push({
        header: '',
        items: Array(targetColumns).fill('')
      })
      continue
    }

    if (index >= data.length) {
      break
    }

    const start = address + row * targetColumns * unitSize

    const element = {
      header: addressString(start),
      items: []
    } as MemoryRow

    for (let column = 0; column < targetColumns; column++) {
      if (index >= data.length) {
        break
      }

      const bytes = data.slice(index, index + unitSize)
      const paddingCount = unitSize - bytes.length
      bytes.push(...Array(paddingCount).fill(0xCC))
      element.items.push(shift(bytes))

      index += unitSize
    }

    result.push(element)
  }

  const header = []

  for (let column = 0; column < targetColumns; column++) {
    header.push(`Value +${(column * unitSize).toString(16)}`)
  }

  return {
    header,
    rows: result
  }
})

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

const modeString = computed(() => {
  switch (state.debug?.mode) {
    case ExecutionMode.Running: return 'Running'
    case ExecutionMode.Breakpoint: return 'Breakpoint'
    case ExecutionMode.Paused: return 'Paused'
    case ExecutionMode.Invalid: return 'Exception'
    case ExecutionMode.BuildFailed: return 'Build Failed'
    default: return 'Debug'
  }
})
const modeClass = computed(() => {
  switch (state.debug?.mode) {
    case ExecutionMode.Running: return 'bg-teal-900'
    case ExecutionMode.Breakpoint: return 'bg-red-900'
    case ExecutionMode.Paused: return 'bg-yellow-900'
    case ExecutionMode.Invalid: return 'bg-red-900'
    case ExecutionMode.BuildFailed: return 'bg-orange-900'
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

async function updateMemoryData() {
  const address = parseAddress(memory.address)

  if (!address) {
    memory.data = []

    return
  }

  if (!state.execution) {
    return
  }

  const maxUnitSize = 4
  const page = targetColumns * targetRows * maxUnitSize

  memory.data = []
  const data = await state.execution.memoryAt(address, page)

  if (!data) {
    return
  }

  memory.data = data.map(x => x ?? 0xCC)
}

const checkMemory = () => {
  if (properties.tab === DebugTab.Memory) {
    updateMemoryData()
  }
}

watch(() => memory.address, checkMemory)
watch(() => properties.tab, checkMemory)
watch(() => state.debug, checkMemory)
</script>
