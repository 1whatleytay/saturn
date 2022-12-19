<template>
  <div class="h-full flex flex-col">
    <TabBar />

    <EditorBody />
  </div>
</template>

<script setup lang="ts">
import TabBar from './TabBar.vue'
import EditorBody from './EditorBody.vue'
</script>
