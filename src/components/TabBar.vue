<template>
  <div>
    <div class="font-mono flex items-center overflow-x-scroll items-start bg-neutral-900 w-full">
      <input type="file" class="hidden" ref="input" @change="loadFile" />

      <Tab
        v-for="tab in state.tabs"
        :key="tab.uuid"
        :tab="tab"
        :selected="state.selected === tab.uuid"
        @select="state.selected = tab.uuid"
        @delete="remove(tab.uuid)"
      />

      <button class="w-10 h-10
          hover:bg-slate-800
          text-slate-300
          flex items-center justify-center
          font-black
        " @click="input?.click()">
        <PlusIcon class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import Tab from './Tab.vue'
import { PlusIcon } from '@heroicons/vue/24/solid'

import { loadElf, remove, state } from '../state/editor-state'

const input = ref(null as HTMLInputElement | null)

function loadFile() {
  const files = input.value?.files

  if (!files?.length) {
    return
  }

  const file = files[0]

  const reader = new FileReader()
  reader.addEventListener('load', () => {
    console.log(reader.result)

    if (reader.result instanceof ArrayBuffer) {
      loadElf(file.name, reader.result)
        .then(() => console.log('Disassembled!'))
    }
  })

  reader.readAsArrayBuffer(file)
}
</script>
