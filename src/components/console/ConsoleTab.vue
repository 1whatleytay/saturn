<template>
  <div
    class="text-xs font-mono overflow-scroll flex flex-col flex-wrap grow whitespace-pre content-start p-2"
  >
    <div v-if="consoleData.console.length">
      {{ consoleData.console }}
    </div>

    <div v-else class="text-neutral-500">
      Nothing yet.
    </div>
  </div>
</template>

<script setup lang="ts">
import { consoleData } from '../../state/console-data'
</script>
