<template>
  <div
    class="font-mono text-sm overflow-scroll flex flex-col flex-wrap grow content-start"
  >
    <div
      v-for="values in registersMap" :key="values[0]"
      class="flex border-b border-neutral-700 w-52"
    >
      <div class="w-20 px-4 py-2 text-neutral-400">
        {{ values[0] }}
      </div>

      <div class="w-32 px-4 py-2 hover:bg-neutral-800 cursor-pointer">
        0x{{ values[1].toString(16) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { consoleData } from '../../state/console-data'

const registers = [
  '$zero', '$at', '$v0', '$v1',
  '$a0', '$a1', '$a2', '$a3',
  '$t0', '$t1', '$t2', '$t3',
  '$t4', '$t5', '$t6', '$t7',
  '$s0', '$s1', '$s2', '$s3',
  '$s4', '$s5', '$s6', '$s7',
  '$t8', '$t9', '$k0', '$k1',
  '$gp', '$sp', '$fp', '$ra',
]

const registersMap = computed(() => {
  const core = registers.map(
    (name, index) => [name, consoleData.debug?.registers[index] ?? 0]
  ) as [string, number][]
  const other = [
    ['hi', consoleData.debug?.hi ?? 0], ['lo', consoleData.debug?.lo ?? 0]
  ] as [string, number][]

  const result = []
  result.push(['pc', consoleData.debug?.pc ?? 0])
  result.push(...core)
  result.push(...other)

  return result
})
</script>
