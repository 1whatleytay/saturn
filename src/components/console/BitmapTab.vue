<template>
  <div
    class="text-sm overflow-auto flex flex-col whitespace-pre content-start p-2"
  >
    <div class="flex items-center text-lg font-bold mb-2 w-full">
      Bitmap Display

      <button
        class="px-2 py-1 border border-neutral-700 rounded text-xs font-medium flex items-center ml-2"
        @click="state.connected = !state.connected"
        :class="{
          'hover:bg-neutral-700': !state.connected,
          'bg-slate-700 hover:bg-slate-600': state.connected
        }"
      >
        <ArrowPathRoundedSquareIcon class="w-4 h-4 mr-1" />

        Connect
      </button>
    </div>

    <button
      ref="wrapper"
      @click="focusSelf"
      @keydown="handleKey"
      class="outline-none focus:ring-4 w-64 h-64 border border-neutral-700 rounded overflow-clip"
    >
      <canvas
        ref="canvas"
        class="w-64 h-64"
        width="256"
        height="256"
      />
    </button>
  </div>
</template>

<script setup lang="ts">
import { onUnmounted, reactive, ref, watch } from 'vue'

import { ArrowPathRoundedSquareIcon } from '@heroicons/vue/24/solid'
import { consoleData } from '../../state/console-data'

const wrapper = ref(null as HTMLElement | null)
const canvas = ref(null as HTMLCanvasElement | null)

const state = reactive({
  connected: false,
  interval: null as number | null
})

async function handleKey(event: KeyboardEvent) {
  if (!consoleData.execution) {
    return
  }

  await consoleData.execution.postKey(event.key)
}

function focusSelf() {
  wrapper.value?.focus()
}

watch(() => state.connected, connect => {
  if (connect) {
    state.interval = window.setInterval(() => {
      reloadDisplay()
    }, 60)
  } else if (state.interval) {
    clearInterval(state.interval)
    state.interval = null
  }
})

onUnmounted(() => {
  if (state.interval) {
    clearInterval(state.interval)
  }
})

async function reloadDisplay() {
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

  const image = await createImageBitmap(data, {
    resizeWidth: 256,
    resizeHeight: 256,
    resizeQuality: 'pixelated'
  })

  context.drawImage(image, 0, 0, 256, 256)
}
</script>
