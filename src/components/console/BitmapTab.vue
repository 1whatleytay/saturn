<template>
  <div
    class="text-sm overflow-auto flex whitespace-pre content-start p-2 h-full"
  >
    <button
      ref="wrapper"
      @click="focusSelf"
      @keydown="handleKey"
      class="outline-none focus:ring-4 border border-neutral-700 rounded overflow-clip h-full shrink-0 mx-auto md:mx-0 max-w-full"
      :style="{ width: `${correctedWidth}px` }"
    >
      <canvas
        ref="canvas"
        class="w-full h-full bitmap-display"
        :width="settings.bitmap.width * 4"
        :height="settings.bitmap.height * 4"
      />
    </button>

    <div class="p-4 hidden md:block flex flex-col content-center justify-center items-center align-center">
      <div class="text-lg font-light mb-4 flex items-center">
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

      <div class="text-neutral-300 ml-2">
        <div class="py-1">
          <label for="bitmap-width" class="inline-block font-bold pr-4 w-32">Display Width</label>
          <NumberField id="bitmap-width" v-model="settings.bitmap.width" />
        </div>

        <div class="py-1">
          <label for="bitmap-height" class="inline-block font-bold pr-4 w-32">Display Height</label>
          <NumberField id="bitmap-height" v-model="settings.bitmap.height" />
        </div>

        <div class="py-1">
          <label for="bitmap-address" class="inline-block font-bold pr-4 w-32">Address</label>
          <NumberField id="bitmap-address" v-model="settings.bitmap.address" :hex="true" />

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

      <div class="text-gray-500 pt-4 flex items-center mt-auto">
        <ExclamationCircleIcon class="w-6 h-6 mr-2" />

        <div v-if="!state.useProtocol">
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

import { ArrowLeftIcon, ArrowPathRoundedSquareIcon } from '@heroicons/vue/24/solid'
import { ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import { consoleData } from '../../state/console-data'

import { settings } from '../../state/state'
import NumberField from './NumberField.vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { ExecutionState } from '../../utils/mips'

const wrapper = ref(null as HTMLElement | null)
const canvas = ref(null as HTMLCanvasElement | null)

const gp = 0x10008000

let lastHeight = settings.bitmap.height
const correctedWidth = ref(settings.bitmap.width)

const state = reactive({
  connected: false,
  interval: null as number | null,
  useProtocol: true
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
})

watch(() => settings.bitmap.width, recheckWidth)
watch(() => settings.bitmap.height, recheckWidth)

watch(() => state.connected, connect => {
  if (connect) {
    state.interval = window.setInterval(() => {
      reloadDisplay()
    }, 60)
  } else if (state.interval) {
    window.clearInterval(state.interval)
    state.interval = null
  }
})

onUnmounted(() => {
  if (state.interval) {
    window.clearInterval(state.interval)
  }
})

async function renderFrameFallback(context: CanvasRenderingContext2D, state: ExecutionState) {
  console.time()
  const memory = await state.memoryAt(0x10008000, 64 * 64 * 4)
  console.timeEnd()

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

async function renderFrameProtocol(context: CanvasRenderingContext2D) {
  const width = settings.bitmap.width
  const height = settings.bitmap.height

  const size = width * height * 4

  const result = await fetch(protocol, {
    headers: {
      width: width.toString(),
      height: height.toString(),
      address: settings.bitmap.address.toString(),
    }
  })

  const memory = new Uint8Array(await result.arrayBuffer())

  const data = context.createImageData(width, height)

  for (let a = 0; a < size; a++) {
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

async function reloadDisplay() {
  const context = canvas.value?.getContext('2d')
  const execution = consoleData.execution

  if (!context || !execution) {
    return
  }

  if (state.useProtocol) {
    try {
      await renderFrameProtocol(context)
      state.useProtocol = false
    } catch {
      // Make up for the lost frame.
      await renderFrameFallback(context, execution)
    }
  } else {
    await renderFrameFallback(context, execution)
  }
}
</script>

<style scoped>
.bitmap-display {
  image-rendering: crisp-edges;
}
</style>
