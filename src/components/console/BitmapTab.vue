<template>
  <div
    class="text-sm overflow-auto flex flex-col whitespace-pre content-start p-2 select-text"
  >
    <div class="flex items-center text-lg font-bold mb-2 w-full">
      Bitmap Display

      <button class="p-1 rounded hover:bg-neutral-700 ml-2" @click="reloadDisplay()">
        <ArrowPathIcon class="w-4 h-4" />
      </button>
    </div>

    <canvas
      ref="canvas"
      class="w-64 h-64 border border-neutral-700 rounded"
      width="256"
      height="256"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import { ArrowPathIcon } from '@heroicons/vue/24/solid'
import { consoleData } from '../../state/console-data'

const canvas = ref(null as HTMLCanvasElement | null)

async function reloadDisplay() {
  const dest = canvas.value
  const context = canvas.value?.getContext('2d')
  const state = consoleData.execution

  if (!context || !state) {
    console.error('No active state or canvas')
    return
  }

  const memory = await state.memoryAt(0x10008000, 64 * 64 * 4)

  if (!memory || memory.length !== 64 * 64 * 4) {
    console.error('No memory at address')
    return
  }

  const mappedMemory = memory
    .map(byte => byte ?? 0)

  const data = context.createImageData(64, 64)

  for (let pixel = 0; pixel < 64 * 64; pixel++) {
    const i = pixel * 4

    data.data[i] = mappedMemory[i + 2]
    data.data[i + 1] = mappedMemory[i + 1]
    data.data[i + 2] = mappedMemory[i]
    data.data[i + 3] = 255
  }

  const image = await createImageBitmap(data)

  context.drawImage(image, 0, 0, 256, 256)
}
</script>
