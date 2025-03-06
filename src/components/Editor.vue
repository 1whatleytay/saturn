<template>
  <div class="flex h-full flex-col">
    <DropZone />

    <TabBar />

    <EditorBody />

    <DebugWindow />
  </div>
</template>

<script setup lang="ts">
import TabBar from './TabBar.vue'
import EditorBody from './EditorBody.vue'
import DebugWindow from './console/Console.vue'
import DropZone from './DropZone.vue'
</script>
