<template>
  <div class="text-sm flex flex-col grow overflow-clip content-start">
    <div class="p-4">
      <div class="border-b text-lg font-semibold">
        Instruction
      </div>

      <div v-if="state.instruction" class="p-2 text-base font-mono select-auto">
        <span class="mr-2">
          Executing:
        </span>

        <span class="text-sky-400 font-bold">
          {{ state.instruction.name }}
        </span>

        <span v-for="(parameter, index) in state.instruction.parameters" :key="index">
          <span v-if="index !== 0">,&nbsp;</span>
          <span v-else class="">&nbsp;</span>

          <span v-if="parameter.type === 'Register'" :class="[registers[parameter.value].color]">
            {{ registers[parameter.value].name }}
          </span>

          <span v-if="parameter.type === 'Address'">0x{{ parameter.value.toString(16) }}</span>

          <span v-if="parameter.type === 'Immediate'">
            {{ parameter.value }}
          </span>
        </span>
      </div>
    </div>

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
import { decodeInstruction, InstructionDetails } from '../../utils/mips'

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

interface StackElement {
  address: number
  value: number
}

const state = reactive({
  values: [] as StackElement[],
  instruction: null as InstructionDetails | null
})

async function currentInstruction(): Promise<InstructionDetails | null> {
  if (!consoleData.registers || !consoleData.execution) {
    return null
  }

  const pc = consoleData.registers.pc

  const data = await consoleData.execution.memoryAt(pc, 4)

  if (data === null) {
    return null
  }

  const instruction = combine(data)

  return await decodeInstruction(pc, instruction)
}

function combine(data: (number | null)[], index: number = 0): number {
  return (data[index] ?? 0)
    + (data[index + 1] ?? 0) * 256
    + (data[index + 2] ?? 0) * 256 * 256
    + (data[index + 3] ?? 0) * 256 * 256 * 256
}

async function stackInfo(): Promise<StackElement[]> {
  const result = [] as StackElement[]

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
    const value = combine(data, index)

    result.push({ address: sp + index, value })
  }

  return result
}

async function loadDetails() {
  state.instruction = await currentInstruction()
  state.values = await stackInfo()
}

watch(
  () => [consoleData.registers, consoleData.execution],
  loadDetails
)

onMounted(loadDetails)
</script>
