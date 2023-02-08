<template>
  <div
    class="text-xs font-mono overflow-hidden flex flex-col grow content-start select-text"
  >
    <div v-if="consoleData.console.length">
      <div
        v-for="item in consoleData.console"
        :key="item.id"
        class="px-2 h-4"
        :class="item.highlight"
      >
        {{ item.text }}
      </div>
    </div>

    <div v-else class="text-neutral-500">
      Nothing yet.
    </div>
  </div>
</template>

<script setup lang="ts">
import { consoleData } from '../../state/console-data'
</script>
