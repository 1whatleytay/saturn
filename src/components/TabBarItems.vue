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

    <button v-if="!!state.execution" class="w-10 h-10
      hover:bg-slate-800
      text-slate-300
      shrink-0
      flex items-center justify-center
      font-black
      text-yellow-200
    " @click="pause()">
      <PauseIcon class="w-4 h-4" />
    </button>

    <button v-if="!!state.execution" class="w-10 h-10
      hover:bg-slate-800
      text-slate-300
      shrink-0
      flex items-center justify-center
      font-black
      text-teal-300
    " @click="step()">
      <ChevronRightIcon class="w-4 h-4" />
    </button>

    <button class="w-10 h-10
      text-slate-300
      shrink-0
      flex items-center justify-center
      font-black
      text-green-300
    " :class="{
      'text-gray-300 cursor-default bg-neutral-800': !allowResume,
      'hover:bg-slate-800': allowResume
    }" @click="resume()" :disabled="!allowResume">
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
      ELF Debug
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { state, tab } from '../state/editor-state'

import { ChevronRightIcon, PlayIcon, PauseIcon, StopIcon } from '@heroicons/vue/24/solid'

import { defaultResult, ExecutionMode, ExecutionState } from '../utils/mips'

const profile = computed(() => tab()?.profile)

const allowResume = computed(() => !state.execution || state.debug?.mode !== ExecutionMode.Running)

async function resume() {
  const usedProfile = tab()?.profile
  const usedBreakpoints = tab()?.breakpoints ?? []

  if (!usedProfile) {
    return
  }

  if (!state.execution) {
    state.execution = new ExecutionState(usedProfile)
  }

  // TODO: On set breakpoint while execution is non-null:
  //  - await state.execution.pause()
  //  - await state.execution.resume(newBreakpoints)

  state.execution.resume(usedBreakpoints)
    .then(result => {
      state.debug = result
    })

  state.debug = defaultResult(ExecutionMode.Running)
}

async function pause() {
  if (state.execution) {
    state.debug = await state.execution.pause()
  }
}

async function step() {
  if (state.execution) {
    state.debug = await state.execution.step()
  }
}

async function stop() {
  if (state.execution) {
    await state.execution.stop()

    state.execution = null
  }
}
</script>
