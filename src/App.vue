<template>
  <div class="h-full">
    <Editor />
  </div>
</template>
<script lang="ts">
import { defineComponent } from 'vue'
import Editor from './components/Editor.vue'

export default defineComponent({
  components: { Editor }
})
</script>
