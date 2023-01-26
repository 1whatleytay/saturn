<template>
  <div v-if="profile ?? false" class="flex items-center">
    <button v-if="!consoleData.execution && tab()?.profile?.kind === 'asm'" class="
      w-10 h-10
      hover:bg-slate-800
      shrink-0
      flex items-center justify-center
      font-black
      text-sky-300
    " @click="build()">
      <ArrowDownIcon class="w-4 h-4" />
    </button>

    <button v-if="!!consoleData.execution" class="
      w-10 h-10
      hover:bg-slate-800
      shrink-0
      flex items-center justify-center
      font-black
      text-red-300
    " @click="stop()">
      <StopIcon class="w-4 h-4" />
    </button>

    <button v-if="!!consoleData.execution" class="
      w-10 h-10
      hover:bg-slate-800
      shrink-0
      flex items-center justify-center
      font-black
      text-yellow-200
    " @click="pause()">
      <PauseIcon class="w-4 h-4" />
    </button>

    <button v-if="!!consoleData.execution" class="
      w-10 h-10
      hover:bg-slate-800
      shrink-0
      flex items-center justify-center
      font-black
      text-teal-300
    " @click="step()">
      <ChevronRightIcon class="w-4 h-4" />
    </button>

    <button class="
      w-10 h-10
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
      shrink-0
    ">
      {{ profileText }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { tab } from '../state/tabs-state'
import { consoleData } from '../state/console-data'
import { build, resume, step, pause, stop } from '../utils/editor-debug'
import { ExecutionModeType } from '../utils/mips'

import {
  ArrowDownIcon,
  PlayIcon,
  PauseIcon,
  StopIcon,
  ChevronRightIcon
} from '@heroicons/vue/24/solid'

const profile = computed(() => tab()?.profile)

const allowResume = computed(() =>
  !consoleData.execution || consoleData.mode !== ExecutionModeType.Running
)

const profileText = computed((): string | null => {
  switch (profile.value?.kind) {
    case 'asm': return 'MIPS Assembly'
    case 'elf': return 'ELF Debug'
  }

  return null
})
</script>
