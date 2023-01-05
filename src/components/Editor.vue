<template>
  <div class="h-full flex flex-col">
    <TabBar />

    <EditorBody />

    <DebugWindow />
  </div>
</template>

<script setup lang="ts">
import TabBar from './TabBar.vue'
import EditorBody from './EditorBody.vue'
import DebugWindow from './Console.vue'
</script>
