<template>
  <div class="flex-auto flex-grow overflow-auto flex px-6 border-l border-neutral-500" :style="{ width: '900px' }">
    <div v-if="buildLines" class="font-mono text-sm w-full">
      <div class="h-6 flex items-center pr-16 text-gray-500 w-full">
        <div class="w-32">
          Address
        </div>

        <div>
          Instruction
        </div>

        <div class="w-32 ml-auto">
          Encoding
        </div>
      </div>

      <div v-for="(line, i) in buildLines" :key="i" class="h-6 flex items-center pr-16 w-full">
        <div v-if="line.type === 'Comment'" class="text-neutral-400 pl-32">
          {{ line.message }}
        </div>

        <div v-if="line.type === 'Label'" class="text-amber-400 pl-32">
          {{ line.name }}:
        </div>

        <div v-if="line.type === 'Instruction'" class="flex w-full">
          <div class="w-32">
            0x{{ line.details.pc.toString(16) }}
          </div>

          <span class="text-sky-400 ml-4">
            {{ line.details.name }}
          </span>

          <span v-for="(parameter, index) in line.details.parameters">
            <span v-if="index !== 0">,&nbsp;</span>
            <span v-else>&nbsp;</span>

            <span v-if="parameter.type === 'Register'" class="text-orange-300">
              {{ registers[parameter.value].name }}
            </span>

            <span v-if="parameter.type === 'Address'" class="text-teal-300">
              0x{{ (parameter.value as number).toString(16) }}
            </span>

            <span v-if="parameter.type === 'Immediate'" class="text-teal-300">
              {{ parameter.value }}
            </span>

            <span v-if="parameter.type === 'Offset'" class="text-teal-300">
              {{ parameter.value.offset }}(<span class="text-orange-300">{{ registers[parameter.value.register].name }}</span>)
            </span>
          </span>

          <div class="w-32 ml-auto">
            0x{{ line.details.instruction.toString(16) }}
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-gray-600">
      Press the build button to generate instructions.
    </div>
  </div>
</template>

<script setup lang="ts">
import { buildLines } from '../state/state'
import { watch } from 'vue'

const systemColor = 'text-purple-300'
const valueColor = 'text-red-300'
const temporaryColor = 'text-cyan-300'
const savedColor = 'text-green-300'

const registers = [
  { name: '$zero', color: systemColor },
  { name: '$at', color: systemColor },
  { name: '$v0', color: valueColor },
  { name: '$v1', color: valueColor },
  { name: '$a0', color: valueColor },
  { name: '$a1', color: valueColor },
  { name: '$a2', color: valueColor },
  { name: '$a3', color: valueColor },
  { name: '$t0', color: temporaryColor },
  { name: '$t1', color: temporaryColor },
  { name: '$t2', color: temporaryColor },
  { name: '$t3', color: temporaryColor },
  { name: '$t4', color: temporaryColor },
  { name: '$t5', color: temporaryColor },
  { name: '$t6', color: temporaryColor },
  { name: '$t7', color: temporaryColor },
  { name: '$s0', color: savedColor },
  { name: '$s1', color: savedColor },
  { name: '$s2', color: savedColor },
  { name: '$s3', color: savedColor },
  { name: '$s4', color: savedColor },
  { name: '$s5', color: savedColor },
  { name: '$s6', color: savedColor },
  { name: '$s7', color: savedColor },
  { name: '$t8', color: temporaryColor },
  { name: '$t9', color: temporaryColor },
  { name: '$k0', color: systemColor },
  { name: '$k1', color: systemColor },
  { name: '$gp', color: systemColor },
  { name: '$sp', color: systemColor },
  { name: '$fp', color: systemColor },
  { name: '$ra', color: systemColor },
]
</script>
