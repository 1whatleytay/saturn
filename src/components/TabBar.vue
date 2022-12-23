<template>
  <div @click.stop>
    <div class="h-10 flex items-center overflow-x-scroll items-start bg-neutral-900 w-full fixed z-20 top-0">
      <input type="file" class="hidden" ref="input" @change="loadFile" />

      <Tab
        v-for="tab in state.tabs"
        :key="tab.uuid"
        :title="tab.title"
        :selected="state.selected === tab.uuid"
        @select="state.selected = tab.uuid"
        @delete="remove(tab.uuid)"
      />

      <button class="w-10 h-10
          hover:bg-slate-800
          text-slate-300
          shrink-0
          flex items-center justify-center
          font-black
        " @click="create">
        <PlusIcon class="w-4 h-4" />
      </button>

      <button class="w-10 h-10
          hover:bg-slate-800
          text-slate-300
          shrink-0
          flex items-center justify-center
          font-black
        " @click="input?.click()">
        <ArrowDownTrayIcon class="w-4 h-4" />
      </button>

      <TabBarItems class="ml-auto" />
    </div>

    <div class="w-full h-10 border-b-2 opacity-0" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

import Tab from './Tab.vue'
import { PlusIcon, ArrowDownTrayIcon } from '@heroicons/vue/24/solid'

import { loadElf, remove, createTab, state } from '../state/editor-state'
import TabBarItems from './TabBarItems.vue'

const input = ref(null as HTMLInputElement | null)

function create() {
  createTab('Untitled', [''])
}

function loadFile() {
  const files = input.value?.files

  if (!files?.length) {
    return
  }

  const file = files[0]

  const reader = new FileReader()
  reader.addEventListener('load', () => {
    if (reader.result instanceof ArrayBuffer) {
      loadElf(file.name, reader.result)
        .then(() => console.log('Disassembled!'))
    }
  })

  reader.readAsArrayBuffer(file)
}
</script>
