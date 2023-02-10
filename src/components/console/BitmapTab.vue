<template>
  <div
    class="text-sm overflow-auto flex whitespace-pre content-start p-2 h-full"
  >
    <button
      ref="wrapper"
      @click="focusSelf"
      @keydown="handleKey"
      class="outline-none overflow-visible focus:ring-4 border border-neutral-700 rounded h-full shrink-0 mx-auto md:mx-0 max-w-full"
      :style="{ width: `${correctedWidth}px` }"
    >
      <canvas
        ref="canvas"
        class="w-full h-full bitmap-display rounded"
        :width="settings.bitmap.width * 4"
        :height="settings.bitmap.height * 4"
      />
    </button>

    <div class="p-4 hidden md:block flex flex-col content-center justify-center items-center align-center">
      <div class="text-lg font-light mb-4 flex items-center">
        Bitmap Display
      </div>

      <div class="text-neutral-300 ml-2">
        <div class="py-1">
          <label for="bitmap-width" class="inline-block font-bold pr-4 w-32">Display Width</label>
          <NumberField id="bitmap-width" v-model="settings.bitmap.width" :checker="sizeCheck" />
        </div>

        <div class="py-1">
          <label for="bitmap-height" class="inline-block font-bold pr-4 w-32">Display Height</label>
          <NumberField id="bitmap-height" v-model="settings.bitmap.height" :checker="sizeCheck" />
        </div>

        <div class="py-1">
          <label for="bitmap-address" class="inline-block font-bold pr-4 w-32">Address</label>
          <NumberField id="bitmap-address" v-model="settings.bitmap.address" :hex="true" :checker="memoryCheck" />

          <button
            class="rounded px-2 py-1 border border-neutral-700 font-bold text-xs ml-4 active:bg-slate-700"
            :class="{
              'bg-slate-800': settings.bitmap.address === gp,
              'hover:bg-slate-800': settings.bitmap.address !== gp
            }"
            @click="settings.bitmap.address = gp"
          >
            $gp
          </button>
        </div>
      </div>

      <div class="text-gray-500 pt-4 flex items-center">
        <ArrowLeftIcon class="w-4 h-4 mr-2" />

        To connect the keyboard, click on the display.
      </div>

      <div v-if="!state.useProtocol" class="text-gray-500 pt-4 flex items-center mt-auto">
        <ExclamationCircleIcon class="w-6 h-6 mr-2" />

        <div>
          <div>
            Using fallback protocol. Performance may be affected.
          </div>

          <div>
            Please file a bug at
            <a
              target="_blank"
              href="https://github.com/1whatleytay/saturn"
              class="underline hover:text-gray-300"
            >https://github.com/1whatleytay/saturn</a>.
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref, watch } from 'vue'

import { ArrowLeftIcon } from '@heroicons/vue/24/solid'
import { ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import { consoleData } from '../../state/console-data'

import { settings } from '../../state/state'
import NumberField from './NumberField.vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { ExecutionState, lastDisplay } from '../../utils/mips'

const wrapper = ref(null as HTMLElement | null)
const canvas = ref(null as HTMLCanvasElement | null)

const gp = 0x10008000

let lastHeight = settings.bitmap.height
const correctedWidth = ref(settings.bitmap.width)

const state = reactive({
  interval: null as number | null,
  useProtocol: true
})

function memoryCheck(value: number): string | null {
  if ((value & 0b11) !== 0) {
    return 'This field must be divisible by 4'
  }

  if (value < 0 || value > 0xffffffff) {
    return 'This field must be in the 32-bit address range'
  }

  return null
}

function sizeCheck(value: number): string | null {
  if (value <= 0 || value > 512) {
    return 'This field must be in the 1-512 range'
  }

  return null
}

async function handleKey(event: KeyboardEvent) {
  if (!consoleData.execution) {
    return
  }

  await consoleData.execution.postKey(event.key)
}

function focusSelf() {
  wrapper.value?.focus()
}

let observer = null as ResizeObserver | null

function fixWidth(height: number) {
  const width = height / settings.bitmap.height * settings.bitmap.width

  lastHeight = height

  // Should not re-trigger this statement.
  correctedWidth.value = width
}

function recheckWidth() {
  if (wrapper.value) {
    fixWidth(wrapper.value.clientHeight)
  }
}

onMounted(() => {
  reloadDisplay()
  checkConnected()

  observer = new ResizeObserver(entries => {
    entries.forEach(entry => {
      if (entry.target.clientHeight != lastHeight) {
        setTimeout(() => fixWidth(entry.target.clientHeight))
      }
    })
  })

  if (wrapper.value) {
    fixWidth(wrapper.value.clientHeight)

    observer.observe(wrapper.value)
  }
})

onUnmounted(() => {
  observer?.disconnect()

  if (state.interval) {
    window.clearInterval(state.interval)
  }
})

watch(() => settings.bitmap.width, recheckWidth)
watch(() => settings.bitmap.height, recheckWidth)

function checkConnected() {
  if (consoleData.execution) {
    state.interval = window.setInterval(() => {
      reloadDisplay()
    }, 60)
  } else {
    reloadDisplay()

    if (state.interval) {
      window.clearInterval(state.interval)
      state.interval = null
    }
  }
}

watch(() => consoleData.execution, checkConnected)

async function renderFrameFallback(context: CanvasRenderingContext2D, execution: ExecutionState) {
  const memory = await execution.memoryAt(0x10008000, 64 * 64 * 4)

  const width = settings.bitmap.width
  const height = settings.bitmap.height

  const pixels = width * height * 4

  if (!memory || memory.length !== pixels) {
    console.error('No memory at address')
    return
  }

  const mappedMemory = memory
    .map(byte => byte ?? 0)

  const data = context.createImageData(width, height)

  for (let pixel = 0; pixel < width * height; pixel++) {
    const i = pixel * 4
    data.data[i] = mappedMemory[i + 2]
    data.data[i + 1] = mappedMemory[i + 1]
    data.data[i + 2] = mappedMemory[i]
    data.data[i + 3] = 255
  }

  const resizeWidth = width * 4
  const resizeHeight = height * 4

  const image = await createImageBitmap(data, {
    resizeWidth, resizeHeight,
    resizeQuality: 'pixelated'
  })

  context.drawImage(image, 0, 0, resizeWidth, resizeHeight)
}

const protocol = convertFileSrc('', 'display')

async function renderOrdered(
  context: CanvasRenderingContext2D,
  width: number, height: number, memory: Uint8Array
) {
  const data = context.createImageData(width, height)

  for (let a = 0; a < memory.length; a++) {
    data.data[a] = memory[a]
  }

  const resizeWidth = width * 4
  const resizeHeight = height * 4

  const image = await createImageBitmap(data, {
    resizeWidth, resizeHeight,
    resizeQuality: 'pixelated'
  })

  context.drawImage(image, 0, 0, resizeWidth, resizeHeight)
}

async function renderFrameProtocol(context: CanvasRenderingContext2D) {
  const width = settings.bitmap.width
  const height = settings.bitmap.height

  const result = await fetch(protocol, {
    headers: {
      width: width.toString(),
      height: height.toString(),
      address: settings.bitmap.address.toString(),
    },
    mode: 'cors',
    cache: 'no-cache'
  })

  const memory = new Uint8Array(await result.arrayBuffer())

  await renderOrdered(context, width, height, memory)
}

async function renderLastDisplay(context: CanvasRenderingContext2D) {
  const last = await lastDisplay()

  // No data, don't render.
  if (!last.data) {
    return
  }

  await renderOrdered(context, last.width, last.height, Uint8Array.from(last.data))
}

async function reloadDisplay() {
  const context = canvas.value?.getContext('2d')
  const execution = consoleData.execution

  if (!context) {
    return
  }

  if (!execution) {
    return await renderLastDisplay(context)
  }

  if (state.useProtocol) {
    try {
      await renderFrameProtocol(context)
    } catch (e) {
      console.error(e)

      state.useProtocol = false
      await renderFrameFallback(context, execution)
    }
  } else {
    await renderFrameFallback(context, execution)
  }
}
</script>

<style scoped>
.bitmap-display {
  image-rendering: pixelated;
}
</style>
