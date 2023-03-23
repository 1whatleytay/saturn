<template>
  <div
    class="text-sm overflow-auto flex whitespace-pre content-start p-2 h-full"
  >
    <button
      ref="wrapper"
      @click="focusSelf"
      @keydown="e => handleKey(e, false)"
      @keyup="e => handleKey(e, true)"
      class="outline-none overflow-visible focus:ring-4 border border-neutral-700 rounded h-full shrink-0 max-w-full"
      :style="{ width: `${correctedWidth}px` }"
      :class="{ 'mx-auto sm:mx-0': !state.small, 'mx-auto': state.small }"
    >
      <canvas
        ref="canvas"
        class="w-full h-full bitmap-display rounded"
        :width="config.width"
        :height="config.height"
      />
    </button>

    <div
      class="p-4 hidden flex flex-col content-center justify-center items-center align-center"
      :class="{ 'sm:block': !state.small }"
    >
      <div class="text-lg font-light mb-4 flex items-center">
        Bitmap Display
      </div>

      <div class="text-neutral-300 ml-2">
        <div class="py-1">
          <label class="inline-block font-bold pr-4 w-32">Display Width</label>

          <NumberField v-model="settings.bitmap.displayWidth" :clean-only="true" :checker="sizeCheck" classes="text-xs w-32" />

          <span class="text-neutral-400 mx-3 text-xs font-bold">
            Units
          </span>

          <NumberField v-model="settings.bitmap.unitWidth" :clean-only="true" :checker="unitCheck" classes="text-xs w-20" />
        </div>

        <div class="py-1">
          <label class="inline-block font-bold pr-4 w-32">Display Height</label>

          <NumberField v-model="settings.bitmap.displayHeight" :clean-only="true" :checker="sizeCheck" classes="text-xs w-32" />

          <span class="text-neutral-400 mx-3 text-xs font-bold">
            Units
          </span>

          <NumberField v-model="settings.bitmap.unitHeight" :clean-only="true" :checker="unitCheck" classes="text-xs w-20" />
        </div>

        <div class="py-1">
          <label class="inline-block font-bold pr-4 w-32">Address</label>
          <NumberField v-model="settings.bitmap.address" :hex="true" :checker="memoryCheck" classes="text-xs w-32" />

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
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'

import { ArrowLeftIcon } from '@heroicons/vue/24/solid'
import { ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import { consoleData } from '../../state/console-data'

import { settings } from '../../state/state'
import NumberField from './NumberField.vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { ExecutionState, lastDisplay } from '../../utils/mips'
import { displayConfig } from '../../utils/settings'

const wrapper = ref(null as HTMLElement | null)
const canvas = ref(null as HTMLCanvasElement | null)

const gp = 0x10008000

const config = computed(() => displayConfig(settings.bitmap))

let lastHeight = config.value.height
const correctedWidth = ref(config.value.width)

const state = reactive({
  interval: null as number | null,
  small: false as boolean,
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

function unitCheck(value: number): string | null {
  if (value <= 0 || value > 32) {
    return 'This field must be in the 1-32 range'
  }

  return null
}

function mapKey(key: string): string | null {
  const lower = key.toLowerCase()

  if (lower.length <= 1) {
    return lower
  } else {
    switch (key) {
      case 'Enter': return '\n'
      case 'Space': return ' '
      default: return null
    }
  }
}

async function handleKey(event: KeyboardEvent, up: boolean) {
  if (!consoleData.execution) {
    return
  }

  const mapped = mapKey(event.key)
  
  if (mapped !== null) {
    await consoleData.execution.postKey(mapped, up)
  }
}

function focusSelf() {
  wrapper.value?.focus()
}

let observer = null as ResizeObserver | null

function fixWidth(height: number) {
  const width = height / config.value.height * config.value.width

  lastHeight = height

  state.small = height < 140

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

watch(() => settings.bitmap, recheckWidth, { deep: true })

function checkConnected() {
  if (consoleData.execution) {
    inflight = false
    state.interval = window.setInterval(() => {
      reloadDisplay()
    }, 20)
  } else {
    inflight = false
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

  const { width, height } = config.value

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

  context.putImageData(data, 0, 0)
}

const protocol = convertFileSrc('', 'display')

function renderOrdered(
  context: CanvasRenderingContext2D,
  width: number, height: number, memory: Uint8Array
) {
  const data = context.createImageData(width, height)

  for (let a = 0; a < memory.length; a++) {
    data.data[a] = memory[a]
  }

  context.putImageData(data, 0, 0)
}

async function renderFrameProtocol(context: CanvasRenderingContext2D) {
  const { width, height } = config.value

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

  renderOrdered(context, width, height, memory)
}

async function renderLastDisplay(context: CanvasRenderingContext2D) {
  const last = await lastDisplay()

  // No data, don't render.
  if (!last.data) {
    return
  }

  await renderOrdered(context, last.width, last.height, Uint8Array.from(last.data))
}

let inflight = false

async function reloadDisplay() {
  if (inflight) {
    return
  }

  inflight = true

  try {
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
  } catch (e) { }

  inflight = false
}
</script>

<style scoped>
.bitmap-display {
  image-rendering: pixelated;
}
</style>
