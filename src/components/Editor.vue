<template>
  <div class="flex h-full flex-col">
    <DropZone />

    <TabBar />

    <EditorBody v-if="tabsState.tabs.length > 0" />
    <div
      v-if="tabsState.tabs.length === 0"
      class="flex h-full items-center justify-center text-2xl font-bold text-slate-800 dark:text-slate-300"
    >
      No tabs open, create a new one!
    </div>

    <DebugWindow />
  </div>
</template>

<script setup lang="ts">
import TabBar from './TabBar.vue'
import EditorBody from './EditorBody.vue'
import DebugWindow from './console/Console.vue'
import DropZone from './DropZone.vue'
import { tabsState } from '../state/state'

</script>
