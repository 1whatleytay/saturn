<template>
  <div class="text-sm flex flex-col grow overflow-hidden content-start">
    <div
      class="flex items-center py-2 border-b border-neutral-700 bg-neutral-900 w-full"
    >
      <label for="address" class="text-xs font-bold px-4 py-2">Address</label>
      <input
        id="address"
        type="text"
        class="text-xs font-mono bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
        v-model="settings.memory.address"
      />

      <div class="flex px-2 space-x-1">
        <button
          class="p-1 rounded hover:bg-neutral-700"
          @click="moveAddress(-1)"
        >
          <ArrowUpIcon class="w-4 h-4" />
        </button>

        <button
          class="p-1 rounded hover:bg-neutral-700"
          @click="moveAddress(+1)"
        >
          <ArrowDownIcon class="w-4 h-4" />
        </button>
      </div>

      <label for="data-type" class="text-xs font-bold px-4 py-2">Type</label>
      <select
        id="data-type"
        class="appearance-none text-xs bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
        :value="settings.memory.mode"
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
          v-for="(item, index) in table.header"
          :key="index"
          class="w-28 flex items-center px-2 py-1 shrink-0"
        >
          {{ item }}
        </div>
      </div>

      <div v-if="table.rows">
        <div
          v-for="(row, index) in table.rows"
          :key="index"
          class="flex font-mono"
        >
          <div class="w-32 px-2 py-1 text-neutral-500 shrink-0">
            {{ row.header }}
          </div>

          <NumberField
            v-for="(item, index) in row.items"
            :key="index"
            :hex="true"
            :model-value="item"
            :bytes="unitSize"
            @update:model-value="value => setAddress(row.address, index, value, settings.memory.mode)"
            classes="bg-transparent hover:bg-neutral-700 w-28 flex items-center px-2 py-1 shrink-0 select-all"
          />
        </div>
      </div>
      <div v-else>
        <div
          class="flex font-mono"
        >
          <div class="w-32 px-2 py-1 text-neutral-500 shrink-0">
            {{ settings.memory.address }}
          </div>

          <div class="font-sans text-neutral-400 flex items-center px-2 py-1 shrink-0 select-all">
            To view memory, set breakpoints or pause during execution.
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { consoleData, DebugTab } from '../../state/console-data'
import { settings } from '../../state/state'
import { AddressingMode } from '../../utils/settings'
import { computed, onMounted, reactive, watch } from 'vue'

import { ArrowDownIcon, ArrowUpIcon } from '@heroicons/vue/24/solid'
import NumberField from './NumberField.vue'

const targetRows = 32
const targetColumns = 8
const defaultAddress = 0x10010000

const memory = reactive({
  data: null as number[] | null,
})

function addressingModeSize(mode: AddressingMode) {
  switch (mode) {
    case AddressingMode.Byte:
      return 1
    case AddressingMode.Half:
      return 2
    case AddressingMode.Word:
      return 4
  }
}

const unitSize = computed(() => addressingModeSize(settings.memory.mode))

function setMode(event: Event) {
  const input = event.target as HTMLInputElement

  try {
    settings.memory.mode = parseInt(input.value) as AddressingMode
  } catch {}
}

function setAddress(start: number, index: number, to: number, mode: AddressingMode) {
  if (!consoleData.execution) {
    return
  }

  const address = start + index * unitSize.value

  switch (mode) {
    case AddressingMode.Byte:
      consoleData.execution.setMemory(address, [to])
      break
    case AddressingMode.Half:
      consoleData.execution.setMemory(address, [to & 0xff, (to >> 8) & 0xff])
      break
    case AddressingMode.Word:
      consoleData.execution.setMemory(address, [
        to & 0xff,
        (to >> 8) & 0xff,
        (to >> 16) & 0xff,
        (to >> 24) & 0xff,
      ])
      break
  }
}

function shift(bytes: number[]): number {
  let result = 0

  for (const byte of bytes.reverse()) {
    // Why not (result << 8) | byte?
    // | and << give signed integers as the result, which isn't what I'm looking for.
    result = (result * 256) + byte
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
  return targetRows * targetColumns * unitSize.value
}

function moveAddress(direction: number) {
  const value = parseAddress(settings.memory.address) ?? defaultAddress
  const section = pageSize()

  settings.memory.address = addressString(value + section * direction)
}

interface MemoryRow {
  header: string
  address: number
  items: number[]
}

interface MemoryTable {
  header: string[]
  rows: MemoryRow[] | null
}

const table = computed((): MemoryTable => {
  const header = []

  const unitSize = addressingModeSize(settings.memory.mode)

  for (let column = 0; column < targetColumns; column++) {
    header.push(`Value +${(column * unitSize).toString(16)}`)
  }

  const address = parseAddress(settings.memory.address)

  if (memory.data === null || address === null) {
    return {
      header,
      rows: null
    }
  }

  const data = memory.data
  let index = 0

  const result = [] as MemoryRow[]

  for (let row = 0; row < targetRows; row++) {
    if (index >= data.length) {
      break
    }

    const start = address + row * targetColumns * unitSize

    const element = {
      header: addressString(start),
      address: start,
      items: [],
    } as MemoryRow

    for (let column = 0; column < targetColumns; column++) {
      if (index >= data.length) {
        break
      }

      const bytes = data.slice(index, index + unitSize)
      const paddingCount = unitSize - bytes.length
      bytes.push(...Array(paddingCount).fill(0xcc))
      element.items.push(shift(bytes))

      index += unitSize
    }

    result.push(element)
  }

  return {
    header,
    rows: result,
  }
})

async function updateMemoryData() {
  const address = parseAddress(settings.memory.address)

  if (!consoleData.execution || !address) {
    memory.data = null

    return
  }

  const maxUnitSize = 4
  const page = targetColumns * targetRows * maxUnitSize

  memory.data = []
  const data = await consoleData.execution.memoryAt(address, page)

  if (!data) {
    memory.data = null

    return
  }

  memory.data = data.map((x) => x ?? 0xcc)
}

const checkMemory = () => {
  if (consoleData.tab === DebugTab.Memory) {
    updateMemoryData()
  }
}

onMounted(updateMemoryData)

watch(() => settings.memory.address, checkMemory)
watch(() => consoleData.tab, checkMemory)
watch(() => consoleData.showConsole, checkMemory)
watch(() => consoleData.registers, checkMemory)
watch(() => consoleData.execution, checkMemory)
</script>
