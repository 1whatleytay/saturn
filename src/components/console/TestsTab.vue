<template>
  <div
    class="text-sm overflow-x-auto flex flex-wrap grow content-start p-4"
  >
    <div class="text-base font-bold mb-4 flex items-center w-full">
      Tests

      <button
        class="ml-auto text-neutral-300 px-4 py-2 rounded-lg flex items-center transition-colors bg-neutral-800 hover:bg-slate-800"
        @click="runTests()"
      >
        <PlayIcon class="text-green-300 font-bold w-4 h-4 mr-2" />

        <span class="text-sm uppercase font-bold">
          Run Tests
        </span>
      </button>
    </div>

    <div class="w-full">
      <div v-if="!state.items.length" class="dark:bg-neutral-800 bg-neutral-300 w-full rounded-lg p-4 flex items-center mb-2">
        <ExclamationCircleIcon class="w-6 h-6 mr-4" />

        No tests configured.
      </div>

      <div
        v-for="item in state.items" :key="item.name"
        class="dark:bg-neutral-800 bg-neutral-300 w-full rounded-lg px-4 py-3 flex items-center mb-2.5"
      >
        <div v-if="item.result === 'Unset'">
          <EllipsisHorizontalCircleIcon class="w-5 h-5 text-blue-400" />
        </div>

        <div v-if="item.result === 'Passed'">
          <CheckCircleIcon class="w-5 h-5 text-green-400 animate-bump" />
        </div>

        <div v-if="item.result === 'Failed'">
          <XCircleIcon class="w-5 h-5 text-red-400 animate-bump" />
        </div>

        <div class="ml-3 font-semibold text-md">
          {{ item.name }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PlayIcon } from '@heroicons/vue/24/solid'
import { EllipsisHorizontalCircleIcon, CheckCircleIcon, XCircleIcon, ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import { onMounted, reactive } from 'vue'
import { invoke } from '@tauri-apps/api'
import { tab } from '../../state/state'

interface TestItem {
  name: string,
  result: string
}

const state = reactive({
  items: [] as TestItem[]
})

onMounted(async () => {
  state.items = await invoke('all_tests')
})

async function runTests() {
  const path = tab()?.path

  if (!path) {
    return
  }

  state.items = await invoke('run_tests', { path })
}
</script>
