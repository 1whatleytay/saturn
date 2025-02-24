<template>
  <Modal :show="props.show" @close="emit('close')">
    <div
      class="pointer-events-auto mx-auto flex max-h-[84vh] max-w-2xl flex-col overflow-y-scroll rounded-xl bg-neutral-200 px-8 py-6 shadow dark:bg-neutral-900"
    >
      <div
        class="my-2 flex w-full shrink-0 items-center bg-neutral-200 text-2xl font-semibold dark:bg-neutral-900"
      >
        <DocumentArrowUpIcon class="mr-3 h-7 w-7 shrink-0" /> Export Regions

        <button
          class="ml-auto flex h-8 w-8 shrink-0 items-center justify-center rounded text-slate-800 hover:bg-slate-300 dark:text-slate-300 dark:hover:bg-slate-800"
          @click="emit('close')"
        >
          <XMarkIcon class="h-4 w-4" />
        </button>
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">Output Format</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          Plain format will export binary, while HexV3 is designed for use with
          Logism Evolution.
        </div>

        <select
          id="data-type"
          class="my-2 w-48 appearance-none rounded bg-neutral-800 px-4 py-2 text-sm font-bold uppercase text-neutral-300"
          :value="state.kind"
          @input="setKind"
        >
          <option value="plain">Plain</option>
          <option value="hex_v3">HexV3</option>
        </select>
      </div>

      <div
        class="mt-8"
        :class="{ 'cursor-not-allowed opacity-50': state.kind !== 'hex_v3' }"
      >
        <div class="text-sm font-bold uppercase">Bit Encoding</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          Encoding type for HexV3 export. For 32-bit memory modules, try using
          32-bit Little Endian.
        </div>

        <select
          id="data-type"
          v-if="state.kind === 'hex_v3'"
          class="my-2 w-48 appearance-none rounded bg-neutral-800 px-4 py-2 text-sm font-bold uppercase text-neutral-300"
          :value="state.encoding"
          :disabled="state.kind !== 'hex_v3'"
          @input="setEncoding"
        >
          <option value="byte">8-bit Encoding</option>
          <option value="little32">32-bit Little Endian</option>
          <option value="big32">32-bit Big Endian</option>
        </select>

        <div
          v-else
          class="my-2 w-48 rounded bg-neutral-800 px-4 py-2 text-sm font-bold uppercase text-neutral-300"
        >
          Plain Encoding
        </div>
      </div>

      <div class="mt-8">
        <div class="text-sm font-bold uppercase">Continuous Export</div>

        <div class="mt-1 text-sm text-gray-800 dark:text-gray-300">
          A continuous export will create one large file with all regions back
          to back.
        </div>

        <ToggleField
          class="my-2"
          title="Continuous Export"
          v-model="state.continuous"
        />
      </div>

      <div class="mt-4 flex items-center">
        <div class="text-sm text-gray-400">
          Continuous exports may result in large file exports (> 800MB).
        </div>

        <button
          class="ml-auto rounded bg-gray-800 px-6 py-3 text-sm font-bold uppercase transition-colors hover:bg-gray-700 active:bg-slate-700"
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
import {
  consoleData,
  ConsoleType,
  DebugTab,
  openConsole,
  pushConsole,
} from '../state/console-data'
import { backend } from '../state/backend'
import {
  exportHexContents,
  exportHexRegions,
} from '../utils/query/serialize-files'
import { postBuildMessage } from '../utils/debug'
import { DocumentArrowUpIcon, XMarkIcon } from '@heroicons/vue/24/solid'
import { settings } from '../state/state'
import ToggleField from './console/ToggleField.vue'
import { toRaw } from 'vue'

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
    pushConsole(
      'Generating hex regions directly from an elf file is not supported.',
      ConsoleType.Info,
    )
    pushConsole(
      'Use an un-assembled assembly file, or submit ' +
        'a feature request at https://github.com/1whatleytay/saturn.',
      ConsoleType.Info,
    )

    return
  }

  const result = await backend.assembleRegions(
    current.doc,
    current.path,
    toRaw(state),
  )

  if (result.regions) {
    switch (result.regions.type) {
      case 'binary': {
        const destination = await exportHexContents(result.regions.value)

        consoleData.showConsole = true
        postBuildMessage(result.result)

        consoleData.tab = DebugTab.Console
        pushConsole(
          `Continuous regions written to ${destination}`,
          ConsoleType.Info,
        )

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
