<template>
  <div
    class="border-10 fixed inset-0 z-[900] flex items-center justify-center border-8 border-dashed border-orange-300 bg-orange-100 bg-opacity-20 text-6xl"
    :class="{ block: isVisible, hidden: !isVisible }"
    @dragenter="show"
    @dragover="show"
    @dragleave="hide"
    @dragend="hide"
    @drop="hide"
  >
    Drop the file to upload it to Saturn
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { createTab } from '../state/state'

const isVisible = ref(false)

const show = (event: DragEvent) => {
  event.preventDefault()

  let isFileTransfer = false
  if (event.dataTransfer?.types) {
    for (var i = 0; i < event.dataTransfer.types.length; i++) {
      if (event.dataTransfer.types[i] == 'Files') {
        isFileTransfer = true
        break
      }
    }
  }

  if (isFileTransfer) isVisible.value = true
}

const hide = async (event: DragEvent) => {
  event.preventDefault()

  isVisible.value = false

  const dataT = event.dataTransfer
  if (!dataT) return

  const items = dataT.files
  if (!items) return

  for (let i = 0; i < items.length; i++) {
    console.log('here', items[i])
    const file = items[i]
    createTab(file.name, await file.text())
  }
}

onMounted(() => {
  window.addEventListener('dragenter', show)

  return () => window.removeEventListener('dragenter', show)
})
</script>
