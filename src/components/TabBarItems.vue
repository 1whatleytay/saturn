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

    <div v-if="profileText" class="
      h-10
      px-4
      flex items-center
      text-xs
      font-medium
      max-w-xs
      text-neutral-600
    ">
      {{ profileText }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { state, tab } from '../state/editor-state'
import { resume, step, pause, stop } from '../state/editor-debug'

import { ChevronRightIcon, PlayIcon, PauseIcon, StopIcon } from '@heroicons/vue/24/solid'

import { ExecutionMode } from '../utils/mips'

const profile = computed(() => tab()?.profile)

const allowResume = computed(() => !state.execution || state.debug?.mode !== ExecutionMode.Running)

const profileText = computed((): string | null => {
  switch (profile.value?.kind) {
    case 'asm': return 'MIPS Assembly'
    case 'elf': return 'ELF Debug'
  }

  return null
})
</script>
