<template>
  <div class="h-full flex flex-col">
    <div class="flex items-center font-mono mx-12 my-8">
      <router-link to="/" class="
        mr-4
        px-2
        py-2
        bg-gray-800
        hover:bg-gray-900
        text-blue-200
        hover:text-light
        rounded
      ">
        <ChevronLeftIcon class="w-4 h-4" />
      </router-link>

      <CpuChipIcon class="w-6 h-6 mr-2" />
<!--      {{ state?.name ?? 'Interpreter' }}-->
    </div>

    <div class="bg-gray-900 flex-grow px-12 pt-4 font-mono">
<!--      {{ text }}-->
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent } from 'vue'
import { CpuChipIcon, ChevronLeftIcon } from '@heroicons/vue/24/solid'

// import { state } from '../state/state'

function binFormat(buffer: ArrayBuffer | null): string {
  if (!buffer) {
    return ''
  }

  const bytes = new Uint8Array(buffer)

  const lines = []
  let line = []
  let index = 0

  console.log(bytes)

  for (const byte of bytes) {
    line.push(byte.toString(16).padStart(2, '0'))

    index += 1

    if (index > 8) {
      lines.push(line.join(' '))
      line = []
    }
  }

  lines.push(line)

  return lines.join('\n')
}

export default defineComponent({
  name: 'Interpreter',

  components: { CpuChipIcon, ChevronLeftIcon },

  setup() {
    return {
      // state,
      //
      // text: computed(() => binFormat(state.value?.blob ?? null))
    }
  }
})
</script>
