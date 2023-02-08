<template>
  <div
    class="text-xs font-mono flex flex-col overflow-hidden grow content-start select-text"
  >
    <div
      ref="scroll"
      class="py-2 overflow-auto w-full h-full"
      v-if="consoleData.console.length"
      @scroll="updateBounds"
    >
      <div :style="{ height: `${topPadding}px` }" />

      <div
        v-for="index in renderCount"
        :key="getIndex(index)"
        class="px-4 h-4"
        :class="[consoleData.console[getIndex(index)].highlight]"
      >
        {{ consoleData.console[getIndex(index)].text }}
      </div>

      <div :style="{ height: `${bottomPadding}px` }" />
    </div>

    <div v-else class="text-neutral-500">
      Nothing yet.
    </div>
  </div>
</template>

<script setup lang="ts">
import { consoleData } from '../../state/console-data'

import { useVirtualize } from '../../utils/virtualization'
import { onMounted, ref, watch } from 'vue'

const lineHeight = 16

const scroll = ref(null as HTMLElement | null)

const {
  renderCount,
  topPadding,
  bottomPadding,
  getIndex,
  update
} = useVirtualize(lineHeight, () => consoleData.console.length)

function updateBounds() {
  if (!scroll.value) {
    return
  }

  update(scroll.value.scrollTop, scroll.value.clientHeight)
}

watch(() => consoleData.console, updateBounds)

onMounted(updateBounds)
</script>
