<template>
  <div v-if="profile ?? false" class="flex items-center">
    <button v-if="!!state.execution" class="w-10 h-10
      hover:bg-slate-800
      text-slate-300
      shrink-0
      flex items-center justify-center
      font-black
      text-red-300
    " @click="stop()">
      <StopIcon class="w-4 h-4" />
    </button>

    <button class="w-10 h-10
      text-slate-300
      shrink-0
      flex items-center justify-center
      font-black
      text-green-300
    " :class="{
      'text-gray-300 cursor-default bg-neutral-800': !!state.execution,
      'hover:bg-slate-800': !state.execution
    }" @click="run()" :disabled="!!state.execution">
      <PlayIcon class="w-4 h-4" />
    </button>

    <div v-if="profile?.elf" class="
      h-10
      px-4
      flex items-center
      text-xs
      font-medium
      max-w-xs
      text-neutral-600
    ">
      ELF Target
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { state, tab } from '../state/editor-state'

import { PlayIcon, StopIcon } from '@heroicons/vue/24/solid'

import { ExecutionResult, ExecutionState } from '../utils/mips'

const profile = computed(() => tab()?.profile)

async function run() {
  if (state.execution) {
    await state.execution.close()
  }

  if (profile.value) {
    state.execution = new ExecutionState(profile.value)

    const result = await state.execution.run(tab()?.breakpoints ?? [])

    state.debug = JSON.stringify(result, null, 2)
  }
}

async function stop() {
  if (state.execution) {
    await state.execution.close()
    state.execution = null
  }
}
</script>
