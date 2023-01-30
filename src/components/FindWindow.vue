<template>
  <div
    v-if="find.state.show"
    class="flex items-center py-2 border-b border-neutral-700 bg-neutral-900 w-full bg-neutral-900 relative z-30"
  >
    <label for="find" class="text-xs font-bold px-4 py-2">Find</label>
    <input
      id="find"
      type="text"
      class="text-xs font-mono bg-neutral-800 text-neutral-300 px-2 py-1 w-40 rounded"
      v-model="find.state.text"
    />

    <div class="flex px-2 space-x-1">
      <button class="p-1 rounded hover:bg-neutral-700">
        <ArrowUpIcon class="w-4 h-4" />
      </button>

      <button class="p-1 rounded hover:bg-neutral-700">
        <ArrowDownIcon class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowUpIcon, ArrowDownIcon } from '@heroicons/vue/24/solid'
import { find } from '../state/state'
</script>
