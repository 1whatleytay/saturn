<template>
  <div @click.stop>
    <div class="h-10 flex items-center overflow-x-auto items-start bg-neutral-900 w-full fixed z-20 top-0">
      <Tab
        v-for="tab in editor.tabs"
        :key="tab.uuid"
        :title="tab.title"
        :marked="tab.marked"
        :selected="editor.selected === tab.uuid"
        :deletable="true"
        @select="editor.selected = tab.uuid"
        @delete="closeTab(tab.uuid)"
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

      <TabBarItems class="ml-auto" />
    </div>

    <div class="w-full h-10 border-b-2 opacity-0" />
  </div>
</template>

<script setup lang="ts">
import Tab from './Tab.vue'
import { PlusIcon, ArrowDownTrayIcon } from '@heroicons/vue/24/solid'

import { loadElf, closeTab, createTab, editor } from '../state/tabs-state'
import TabBarItems from './TabBarItems.vue'

function create() {
  createTab('Untitled', [''])
}
</script>
