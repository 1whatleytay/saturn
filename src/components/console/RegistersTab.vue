<template>
  <div
    class="font-mono text-sm overflow-x-auto flex flex-wrap grow content-start"
  >
    <div
      v-for="section of mappedSections"
      :key="section.name"
      class="px-6 w-full mb-2"
    >
      <div :class="section.classes" class="flex items-center font-sans mt-2 mb-1 pb-1 text-lg border-b border-gray-700 font-light w-full">
        <Square3Stack3DIcon class="w-4 h-4 mr-2" />

        {{ section.name }}
      </div>

      <div class="flex items-center flex-wrap w-full">
        <div
          v-for="register of section.values"
          :key="register.name"
          class="w-28 p-1 h-12"
        >
          <div class="text-xs mb-1" :class="[section.classes, register.value === undefined ? 'opacity-50' : '']">
            {{ register.name }}
          </div>

          <div v-if="register.value !== undefined" class="text-gray-100">
            0x{{ register.value?.toString(16) }}
          </div>
        </div>
      </div>
    </div>

    <div class="w-full mb-16" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { consoleData } from '../../state/console-data'

import { Square3Stack3DIcon } from '@heroicons/vue/24/outline'

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

interface RegisterValue {
  name: string
  value?: number
}

interface OutlinedSection {
  name: string,
  classes: string
  indices: number[]
  system: boolean
}

const sections = [
  { name: 'System', classes: 'text-purple-300', indices: [0, 1, 26, 27, 28, 29, 30, 31], system: true },
  { name: 'Values', classes: 'text-red-300', indices: [2, 3, 4, 5, 6, 7] },
  { name: 'Temporary', classes: 'text-cyan-300', indices: [8, 9, 10, 11, 12, 13, 14, 15, 24, 25] },
  { name: 'Saved', classes: 'text-green-300', indices: [16, 17, 18, 19, 20, 21, 22, 23] },
] as OutlinedSection[]

interface RegisterSection {
  name: string,
  classes: string
  values: RegisterValue[]
}

const mappedSections = computed((): RegisterSection[] => {
  const system = [
    { name: 'pc', value: consoleData.registers?.pc },
    { name: 'hi', value: consoleData.registers?.hi },
    { name: 'lo', value: consoleData.registers?.lo },
  ] as RegisterValue[]

  return sections.map(section => {
    const values = section.system ? system.slice() : []

    values.push(...section.indices.map(index => ({
      name: registers[index],
      value: consoleData.registers?.line[index]
    })))

    return {
      name: section.name,
      classes: section.classes,
      values
    }
  })
})
</script>
