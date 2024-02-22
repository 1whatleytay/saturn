<template>
  <div class="h-full flex flex-col">
    <TabBar />

    <FindWindow />

    <EditorBody />

    <DebugWindow />
  </div>
</template>

<script setup lang="ts">
import TabBar from './TabBar.vue'
import EditorBody from './EditorBody.vue'
import DebugWindow from './console/Console.vue'
import FindWindow from './FindWindow.vue'
import BuildInfo from './BuildInfo.vue'
</script>
