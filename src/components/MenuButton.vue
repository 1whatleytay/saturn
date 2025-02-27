<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger>
      <button
        class="flex h-10 w-10 shrink-0 items-center justify-center font-black text-slate-800 hover:bg-slate-300 dark:text-slate-300 dark:hover:bg-slate-800"
      >
        <Bars3Icon class="h-4 w-4" />
      </button>
    </DropdownMenuTrigger>

    <DropdownMenuPortal>
      <DropdownMenuContent
        class="ml-2 w-60 gap-2 rounded-lg rounded-tl-none border bg-neutral-200 p-2 text-sm text-slate-800 shadow-md dark:border-neutral-700 dark:bg-neutral-900 dark:text-slate-300"
      >
        <DropdownMenuItem
          @click="showSettings = true"
          value="Settings"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <CogIcon class="m-2 h-4 w-4" />
          Settings
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+,</div>
        </DropdownMenuItem>

        <DropdownMenuSeparator
          class="my-2 border-t border-neutral-300 dark:border-neutral-700"
        />

        <!-- New tab, open file, close tab -->
        <DropdownMenuItem
          @click="emit('create')"
          value="New Tab"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <FolderPlusIcon class="m-2 h-4 w-4" />
          New Tab
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+T</div>
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="openFile"
          value="Open File"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <DocumentArrowDownIcon class="m-2 h-4 w-4" />
          Open File
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+O</div>
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="saveCurrentTab"
          value="Save File"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <DocumentArrowUpIcon class="m-2 h-4 w-4" />
          Save File
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+S</div>
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="download"
          value="Download File"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <DocumentArrowUpIcon class="m-2 h-4 w-4" />
          Download File
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+⇧+D</div>
        </DropdownMenuItem>

        <DropdownMenuSeparator
          class="my-2 border-t border-neutral-300 dark:border-neutral-700"
        />

        <DropdownMenuItem
          @click="build"
          value="Build"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <CogIcon class="m-2 h-4 w-4" />
          Build
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+B</div>
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="resume"
          value="Run"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <CogIcon class="m-2 h-4 w-4" />
          Run
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+K</div>
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="step"
          value="Step"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <CogIcon class="m-2 h-4 w-4" />
          Step
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+L</div>
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="pause"
          value="Pause"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <CogIcon class="m-2 h-4 w-4" />
          Pause
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+J</div>
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="stop"
          value="Stop"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <CogIcon class="m-2 h-4 w-4" />
          Stop
          <div class="ml-auto mr-1 pl-[20px] font-mono">⌘+P</div>
        </DropdownMenuItem>

        <DropdownMenuSeparator
          class="my-2 border-t border-neutral-300 dark:border-neutral-700"
        />

        <!-- Assemble elf, disassemble elf, export elf, export regions -->
        <DropdownMenuItem
          @click="assemble"
          value="Assemble ELF"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <ArrowUpLeftIcon class="m-2 h-4 w-4" />
          Assemble ELF
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="disassemble"
          value="Disassemble ELF"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <ArrowUpLeftIcon class="m-2 h-4 w-4" />
          Disassemble ELF
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="exportBinary"
          value="Export ELF"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <ArrowUpLeftIcon class="m-2 h-4 w-4" />
          Export ELF
        </DropdownMenuItem>

        <DropdownMenuItem
          @click="exportHex"
          value="Export Regions"
          class="flex items-center rounded hover:bg-neutral-300 dark:hover:bg-neutral-700"
        >
          <ArrowUpLeftIcon class="m-2 h-4 w-4" />
          Export Regions
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>

<script setup lang="ts">
import {
  Bars3Icon,
  CogIcon,
  ArrowUpLeftIcon,
  FolderPlusIcon,
  DocumentArrowDownIcon,
  DocumentArrowUpIcon,
} from '@heroicons/vue/24/solid'
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from 'reka-ui'
import { showSettings } from '../state/state'
import { build, pause, resume, step, stop } from '../utils/debug'
import {
  assemble,
  disassemble,
  exportHex,
  openFile,
  saveCurrentTab,
} from '../utils/events/events'
import { exportBinary, download } from '../utils/events/web-shortcuts'

const emit = defineEmits(['create'])
</script>
