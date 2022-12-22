<template>
  <div v-if="state.debug">
    <div class="w-full" :style="{ height: `${properties.height}px` }" />

    <div
      class="
        w-full
        fixed
        bottom-0
        z-30
        bg-neutral-800
        border-t border-neutral-700
        flex flex-col
      " :style="{ height: `${properties.height}px` }">
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

        Run Results
      </div>

      <div class="font-mono whitespace-pre text-sm overflow-y-scroll grow pt-4 px-10">
        {{ state.debug }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { state } from '../state/editor-state'

import { MinusIcon } from '@heroicons/vue/24/solid'
import { reactive } from 'vue'

const properties = reactive({
  height: 320
})

function close() {
  state.debug = null
}
</script>
