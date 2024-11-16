<template>
  <div class="h-full select-none">
    <Editor />
  </div>
</template>

<script setup lang="ts">
import Editor from './components/Editor.vue'
import { onErrorCaptured } from 'vue'
import { invoke } from '@tauri-apps/api'

onErrorCaptured((err, instance, info) => {
  invoke('send_trace', { text: `![VUE ERROR]! ${err.name}, ${err.message}, ${instance?.$el}, ${info}` })
    .then(() => { })
})
</script>
