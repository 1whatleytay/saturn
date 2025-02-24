<template>
  <div class="flex grow flex-wrap content-start overflow-x-auto p-4 text-sm">
    <div class="mb-4 flex w-full items-center text-base font-bold">
      Tests

      <button
        class="ml-auto flex items-center rounded-lg bg-neutral-800 px-4 py-2 text-neutral-300 transition-colors hover:bg-slate-800"
        @click="runTests()"
      >
        <PlayIcon class="mr-2 h-4 w-4 font-bold text-green-300" />

        <span class="text-sm font-bold uppercase"> Run Tests </span>
      </button>
    </div>

    <div class="w-full">
      <div
        v-if="!state.items.length"
        class="mb-2 flex w-full items-center rounded-lg bg-neutral-300 p-4 dark:bg-neutral-800"
      >
        <ExclamationCircleIcon class="mr-4 h-6 w-6" />

        No tests configured.
      </div>

      <div
        v-for="item in state.items"
        :key="item.name"
        class="mb-2.5 flex w-full items-center rounded-lg bg-neutral-300 px-4 py-3 dark:bg-neutral-800"
      >
        <div v-if="item.result === 'Unset'">
          <EllipsisHorizontalCircleIcon class="h-5 w-5 text-blue-400" />
        </div>

        <div v-if="item.result === 'Passed'">
          <CheckCircleIcon class="h-5 w-5 animate-bump text-green-400" />
        </div>

        <div v-if="item.result === 'Failed'">
          <XCircleIcon class="h-5 w-5 animate-bump text-red-400" />
        </div>

        <div class="text-md ml-3 font-semibold">
          {{ item.name }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PlayIcon } from '@heroicons/vue/24/solid'
import {
  EllipsisHorizontalCircleIcon,
  CheckCircleIcon,
  XCircleIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'
import { onMounted, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { tab } from '../../state/state'

interface TestItem {
  name: string
  result: string
}

const state = reactive({
  items: [] as TestItem[],
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
