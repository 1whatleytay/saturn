<template>
  <Modal :show="props.show" @close="emit('close')">
    <div
      class="max-w-2xl bg-neutral-900 rounded-xl px-8 py-6 mx-auto flex flex-col shadow pointer-events-auto overflow-y-scroll max-h-[84vh]"
    >
      <div class="text-2xl font-semibold flex items-center bg-neutral-900 w-full my-2 shrink-0">
        <DocumentArrowUpIcon class="w-7 h-7 mr-3 shrink-0" /> Export Regions

        <button
          class="w-8 h-8 ml-auto rounded hover:bg-slate-800 text-slate-300 shrink-0 flex items-center justify-center"
          @click="emit('close')"
        >
          <XMarkIcon class="w-4 h-4" />
        </button>
      </div>

      <div class="mt-8">
        <div class="font-bold uppercase text-sm">
          Output Format
        </div>

        <div class="text-gray-300 text-sm mt-1">
          Plain format will export binary,
          while HexV3 is designed for use with Logism Evolution.
        </div>

        <select
          id="data-type"
          class="appearance-none uppercase font-bold text-sm bg-neutral-800 text-neutral-300 px-4 py-2 my-2 w-48 rounded"
          :value="state.kind"
          @input="setKind"
        >
          <option value="plain">Plain</option>
          <option value="hex_v3">HexV3</option>
        </select>
      </div>

      <div class="mt-8" :class="{'opacity-50 cursor-not-allowed': state.kind !== 'hex_v3'}">
        <div class="font-bold uppercase text-sm">
          Bit Encoding
        </div>

        <div class="text-gray-300 text-sm mt-1">
          Encoding type for HexV3 export.
          For 32-bit memory modules, try using 32-bit Little Endian.
        </div>

        <select
          id="data-type"
          v-if="state.kind === 'hex_v3'"
          class="appearance-none uppercase font-bold text-sm bg-neutral-800 text-neutral-300 px-4 py-2 my-2 w-48 rounded"
          :value="state.encoding"
          :disabled="state.kind !== 'hex_v3'"
          @input="setEncoding"
        >
          <option value="byte">8-bit Encoding</option>
          <option value="little32">32-bit Little Endian</option>
          <option value="big32">32-bit Big Endian</option>
        </select>

        <div v-else class="uppercase font-bold text-sm bg-neutral-800 text-neutral-300 px-4 py-2 my-2 w-48 rounded">
          Plain Encoding
        </div>
      </div>

      <div class="mt-8">
        <div class="font-bold uppercase text-sm">
          Continuous Export
        </div>

        <div class="text-gray-300 text-sm mt-1">
          A continuous export will create one large file with all regions back to back.
        </div>

        <ToggleField
          class="my-2"
          title="Continuous Export"
          v-model="state.continuous"
        />
      </div>

      <div class="flex items-center mt-4">
        <div class="text-sm text-gray-400">
          Continuous exports may result in large file exports (> 800MB).
        </div>

        <button
          class="rounded px-6 py-3 bg-gray-800 hover:bg-gray-700 transition-colors uppercase font-bold text-sm ml-auto active:bg-slate-700"
          @click="exportRegions()"
        >
          Export
        </button>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import Modal from './Modal.vue'

import { tab } from '../state/state'
import { consoleData, ConsoleType, DebugTab, openConsole, pushConsole } from '../state/console-data'
import { backend } from '../state/backend'
import { collectLines } from '../utils/tabs'
import { exportHexContents, exportHexRegions } from '../utils/query/serialize-files'
import { postBuildMessage } from '../utils/debug'
import { DocumentArrowUpIcon, XMarkIcon } from '@heroicons/vue/24/solid'
import { settings } from '../state/state'
import ToggleField from './console/ToggleField.vue'

const props = defineProps<{
  show: boolean
}>()

function setEncoding(event: Event) {
  const value = (event.target as HTMLSelectElement).value

  if (value !== 'byte' && value !== 'big32' && value !== 'little32') {
    return
  }

  state.encoding = value
}

function setKind(event: Event) {
  const value = (event.target as HTMLSelectElement).value

  if (value !== 'plain' && value !== 'hex_v3') {
    return
  }

  state.kind = value
}

const state = settings.export

const emit = defineEmits(['close'])

async function exportRegions() {
  emit('close')

  const current = tab()

  if (!current) {
    return
  }

  if (current.profile && current.profile.kind !== 'asm') {
    consoleData.showConsole = true

    openConsole()
    pushConsole('Generating hex regions directly from an elf file is not supported.', ConsoleType.Info)
    pushConsole('Use an un-assembled assembly file, or submit ' +
      'a feature request at https://github.com/1whatleytay/saturn.', ConsoleType.Info)

    return
  }

  const result = await backend.assembleRegions(collectLines(current.lines), current.path, state)

  if (result.regions) {
    switch (result.regions.type) {
      case 'binary': {
        const destination = await exportHexContents(result.regions.value)

        consoleData.showConsole = true
        postBuildMessage(result.result)

        consoleData.tab = DebugTab.Console
        pushConsole(`Continuous regions written to ${destination}`, ConsoleType.Info)

        break
      }

      case 'split': {
        const destination = await exportHexRegions(result.regions.value)

        consoleData.showConsole = true
        postBuildMessage(result.result)

        consoleData.tab = DebugTab.Console
        pushConsole(`Regions data written to ${destination}`, ConsoleType.Info)

        break
      }
    }
  }
}
</script>
