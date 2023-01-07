<template>
  <div
    class="text-sm flex flex-col grow overflow-clip content-start"
  >
    <div class="flex items-center py-2 border-b border-neutral-700 bg-neutral-900 w-full">
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

    <div class="text-right pt-4 overflow-auto grow shrink flex flex-col">
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
</template>

<script setup lang="ts">
import { consoleData, DebugTab } from '../../state/console-data'
import { computed, reactive, watch } from 'vue'

import { ArrowDownIcon, ArrowUpIcon } from '@heroicons/vue/24/solid'

enum AddressingMode {
  Byte,
  Half,
  Word,
}

const targetRows = 32
const targetColumns = 8
const defaultAddress = 0x10010000

const memory = reactive({
  address: addressString(defaultAddress),
  data: Array(256).fill(0),
  mode: AddressingMode.Word
})

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

async function updateMemoryData() {
  const address = parseAddress(memory.address)

  if (!address) {
    memory.data = []

    return
  }

  if (!consoleData.execution) {
    return
  }

  const maxUnitSize = 4
  const page = targetColumns * targetRows * maxUnitSize

  memory.data = []
  const data = await consoleData.execution.memoryAt(address, page)

  if (!data) {
    return
  }

  memory.data = data.map(x => x ?? 0xCC)
}

const checkMemory = () => {
  if (consoleData.tab === DebugTab.Memory) {
    updateMemoryData()
  }
}

watch(() => memory.address, checkMemory)
watch(() => consoleData.tab, checkMemory)
watch(() => consoleData.showConsole, checkMemory)
watch(() => consoleData.registers, checkMemory)
</script>
