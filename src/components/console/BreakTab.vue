<template>
  <div class="text-sm flex flex-col grow overflow-clip content-start">
    <div class="p-4">
      Stack

      <div v-for="value in state.values" class="flex items-center font-mono">
        <div class="w-32">
          0x{{ value.address.toString(16).padStart(8, '0') }}
        </div>

        <div class="w-32">
          0x{{ value.value.toString(16).padStart(8, '0') }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { consoleData } from '../../state/console-data'
import { onMounted, reactive, watch } from 'vue'

const state = reactive({
  values: [] as { address: number, value: number }[]
})

async function stackInfo(): Promise<{ address: number, value: number }[]> {
  const result = [] as { address: number, value: number }[]

  if (!consoleData.registers || !consoleData.execution) {
    return result
  }

  const sp = consoleData.registers.line[29]

  const data = await consoleData.execution.memoryAt(sp, 4 * 12)

  if (!data) {
    return result
  }

  for (let index = 0; index < data.length; index += 4) {
    // Avoid << because we don't want signed integers
    const value = (data[index] ?? 0)
      + (data[index + 1] ?? 0) * 256
      + (data[index + 2] ?? 0) * 256 * 256
      + (data[index + 3] ?? 0) * 256 * 256 * 256

    result.push({ address: sp + index, value })
  }

  return result
}

watch(
  () => [consoleData.registers, consoleData.execution],
  async () => state.values = await stackInfo()
)
</script>
