<template>
  <div v-if="state.debug">
    <div class="w-full" :style="{ height: `${properties.height}px` }" />

    <div
      ref="grabber"
      class="
        w-full
        fixed
        bottom-0
        z-30
        bg-neutral-900
        border-t border-neutral-700
        flex flex-col
      " :style="{
        height: `${properties.height}px`,
        opacity: properties.height > closingHeight ? '1' : '0'
      }">
      <div
        class="absolute top-0 h-2 w-full cursor-row-resize"
        @mousedown="handleDown"
      />

      <div class="h-10 mb-2 flex items-center text-sm font-bold text-neutral-400">
        <button class="w-10 h-10 mr-4
            hover:bg-slate-800
            text-slate-300
            shrink-0
            flex items-center justify-center
            font-black
          " @click="close">
          <MinusIcon class="w-4 h-4" />
        </button>

        Debug
      </div>

      <div class="font-mono whitespace-pre text-sm overflow-y-scroll grow pt-4 px-10">
        {{ state.debug }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref, watch } from 'vue'

import { state } from '../state/editor-state'

import { MinusIcon } from '@heroicons/vue/24/solid'

const closingHeight = 90
const defaultHeight = 320

const properties = reactive({
  lastHeight: defaultHeight,
  height: defaultHeight,
  resizing: null as number | null
})

const grabber = ref(null as HTMLElement | null)

function close() {
  state.debug = null
}

function grabberPosition(event: MouseEvent): { x: number, y: number } {
  return {
    x: event.pageX - (grabber.value?.offsetLeft ?? 0),
    y: event.pageY - (grabber.value?.offsetTop ?? 0)
  }
}

function handleDown(event: MouseEvent) {
  const position = grabberPosition(event)

  if (!position) {
    return
  }

  properties.resizing = position.y
}

const handleMove = (event: MouseEvent) => {
  // properties.resizing = posY
  if (!properties.resizing) {
    return
  }

  if ((event.buttons & 1) === 0) {
    properties.resizing = null

    return
  }

  const position = grabberPosition(event)

  if (!position) {
    return
  }

  const difference = properties.resizing - position.y

  properties.height += difference
}

const handleUp = (event: MouseEvent) => {
  properties.resizing = null
}

onMounted(() => {
  window.addEventListener('mousemove', handleMove)
  window.addEventListener('mouseup', handleUp)
})

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMove)
  window.removeEventListener('mouseup', handleUp)
})

function watchHeight(height: number, resizing: boolean) {
  if (resizing) {
    return
  }

  if (height < closingHeight) {
    state.debug = null

    properties.height = properties.lastHeight
  } else {
    properties.lastHeight = height
  }
}

watch(() => properties.height, height => {
  watchHeight(height, !!properties.resizing)
})

watch(() => properties.resizing, resizing => {
  watchHeight(properties.height, !!resizing)
})
</script>
