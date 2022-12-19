<template>
  <div class="h-full flex flex-col">
    <TabBar />

    <div class="font-mono text-sm pt-4 flex-auto flex-grow overflow-scroll">
      <div v-for="(line, index) in tab()?.lines ?? ['Nothing yet.']"  class="pl-1 py-1.5 flex items-center">
        <div class="w-8 text-right mr-6 text-xs text-slate-600">{{ index + 1 }}</div>

        {{ line }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { tab } from '../state/editor-state'

import TabBar from './TabBar.vue'
</script>
