<template>
  <div v-if="profile ?? false" class="flex items-center">
    <button
      v-if="!consoleData.execution && tab()?.profile?.kind === 'asm'"
      class="w-10 h-10 hover:bg-slate-800 shrink-0 flex items-center justify-center font-black text-sky-300"
      @click="build()"
      title="Build"
    >
      <ArrowDownIcon class="w-4 h-4" />
    </button>

    <button
      v-if="!!consoleData.execution"
      class="w-10 h-10 hover:bg-slate-800 shrink-0 flex items-center justify-center font-black text-red-300"
      @click="stop()"
      title="Stop"
    >
      <StopIcon class="w-4 h-4" />
    </button>

    <button
      v-if="!!consoleData.execution"
      class="w-10 h-10 hover:bg-slate-800 shrink-0 flex items-center justify-center font-black text-yellow-200"
      @click="pause()"
      title="Pause"
    >
      <PauseIcon class="w-4 h-4" />
    </button>

    <button
      v-if="!!consoleData.execution && consoleData.execution.timeTravel"
      class="w-10 h-10 shrink-0 flex items-center justify-center font-black"
      @click="rewind()"
      :class="{
        'text-gray-300 cursor-default': !allowRewind,
        'text-teal-300 hover:bg-slate-800': allowRewind,
      }"
      :disabled="!allowRewind"
      title="Step Back"
    >
      <ChevronLeftIcon class="w-4 h-4" />
    </button>

    <button
      v-if="!!consoleData.execution"
      class="w-10 h-10 shrink-0 flex items-center justify-center font-black"
      @click="step()"
      :class="{
        'text-gray-300 cursor-default': !allowResume,
        'text-teal-300 hover:bg-slate-800': allowResume,
      }"
      :disabled="!allowResume"
      title="Step"
    >
      <ChevronRightIcon class="w-4 h-4" />
    </button>

    <button
      class="w-10 h-10 shrink-0 flex items-center justify-center font-black"
      :class="{
        'text-gray-300 cursor-default bg-neutral-800': !allowResume,
        'text-green-300 hover:bg-slate-800': allowResume,
      }"
      @click="resume()"
      :disabled="!allowResume"
      title="Run"
    >
      <PlayIcon class="w-4 h-4" />
    </button>

    <div
      v-if="profileText"
      class="h-10 px-4 flex items-center text-xs font-medium max-w-xs text-neutral-600 shrink-0"
    >
      {{ profileText }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { consoleData } from '../state/console-data'
import { build, pause, resume, step, rewind, stop } from '../utils/debug'
import { ExecutionModeType } from '../utils/mips/mips'
import { tab } from '../state/state'

import {
  ArrowDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  PauseIcon,
  PlayIcon,
  StopIcon,
} from '@heroicons/vue/24/solid'

const profile = computed(() => tab()?.profile)

const allowRewind = computed(
  () =>
    !consoleData.execution || (consoleData.mode !== ExecutionModeType.Running)
)

const allowResume = computed(
  () =>
    !consoleData.execution ||
    (consoleData.mode !== ExecutionModeType.Invalid &&
      consoleData.mode !== ExecutionModeType.Running)
)

const profileText = computed((): string | null => {
  switch (profile.value?.kind) {
    case 'asm':
      return 'MIPS Assembly'
    case 'elf':
      return 'ELF Debug'
  }

  return null
})
</script>
