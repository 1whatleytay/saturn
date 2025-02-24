<template>
  <div v-if="profile ?? false" class="flex items-center">
    <button
      v-if="!consoleData.execution && tab()?.profile?.kind === 'asm'"
      class="flex h-10 w-10 shrink-0 items-center justify-center font-black text-sky-700 hover:bg-slate-300 dark:text-sky-300 dark:hover:bg-slate-800"
      @click="build()"
      title="Build"
    >
      <ArrowDownIcon class="h-4 w-4" />
    </button>

    <button
      v-if="!!consoleData.execution"
      class="flex h-10 w-10 shrink-0 items-center justify-center font-black text-red-700 hover:bg-slate-300 dark:text-red-300 dark:hover:bg-slate-800"
      @click="stop()"
      title="Stop"
    >
      <StopIcon class="h-4 w-4" />
    </button>

    <button
      v-if="!!consoleData.execution"
      class="flex h-10 w-10 shrink-0 items-center justify-center font-black text-yellow-800 hover:bg-slate-300 dark:text-yellow-200 dark:hover:bg-slate-800"
      @click="pause()"
      title="Pause"
    >
      <PauseIcon class="h-4 w-4" />
    </button>

    <button
      v-if="!!consoleData.execution && consoleData.execution.timeTravel"
      class="flex h-10 w-10 shrink-0 items-center justify-center font-black"
      @click="rewind()"
      :class="{
        'cursor-default text-gray-700 dark:text-gray-300': !allowRewind,
        'text-teal-700 hover:bg-slate-300 dark:text-teal-300 dark:hover:bg-slate-800':
          allowRewind,
      }"
      :disabled="!allowRewind"
      title="Step Back"
    >
      <ChevronLeftIcon class="h-4 w-4" />
    </button>

    <button
      v-if="!!consoleData.execution"
      class="flex h-10 w-10 shrink-0 items-center justify-center font-black"
      @click="step()"
      :class="{
        'cursor-default text-gray-700 dark:text-gray-300': !allowResume,
        'text-teal-700 hover:bg-slate-300 dark:text-teal-300 dark:hover:bg-slate-800':
          allowResume,
      }"
      :disabled="!allowResume"
      title="Step"
    >
      <ChevronRightIcon class="h-4 w-4" />
    </button>

    <button
      class="flex h-10 w-10 shrink-0 items-center justify-center font-black"
      :class="{
        'cursor-default bg-neutral-400 text-gray-700 dark:bg-neutral-800 dark:text-gray-300':
          !allowResume,
        'text-green-700 hover:bg-slate-300 dark:text-green-300 dark:hover:bg-slate-800':
          allowResume,
      }"
      @click="resume()"
      :disabled="!allowResume"
      title="Run"
    >
      <PlayIcon class="h-4 w-4" />
    </button>

    <div
      v-if="profileText"
      class="flex h-10 max-w-xs shrink-0 items-center px-4 text-xs font-medium text-neutral-600"
    >
      {{ profileText }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { consoleData } from '../state/console-data'
import {
  build,
  pause,
  resume,
  step,
  rewind,
  stop,
  allowResume,
  allowRewind,
} from '../utils/debug'
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
