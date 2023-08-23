<template>
  <div class="text-sm flex flex-col grow overflow-clip content-start overflow-y-scroll">
    <div class="text-2xl font-bold w-full px-8 py-4">
      {{ consoleData.mode ?? 'Debug' }}
    </div>

    <div class="flex">
      <div v-if="!state.instructions && !state.stack" class="px-8 text-neutral-500">
        Run the application to show things on the debug tab.
      </div>

      <div class="w-1/2 px-8" v-if="state.instructions">
        <div class="text-base select-auto mt-3">
          <div class="text-lg font-semibold mb-2">
            Instruction
          </div>

          <div class="font-mono my-1" v-for="(instruction, index) in state.instructions.instructions" :key="index">
            <div
              class="bg-blue-500 rounded-full w-3 h-3 mr-2 inline-block"
              :class="{'opacity-0': state.instructions.currentIndex !== index}"
            />

            <span class="text-sky-400 font-bold">
              {{ instruction?.name || 'unk' }}
            </span>

            <span v-for="(parameter, index) in instruction?.parameters ?? []" :key="index">
              <span v-if="index !== 0">,&nbsp;</span>
              <span v-else>&nbsp;</span>

              <span v-if="parameter.type === 'Register'" :class="[registers[parameter.value].color]">
                {{ registers[parameter.value].name }}
              </span>

              <span v-if="parameter.type === 'Address'">
                0x{{ parameter.value.toString(16) }}
              </span>

              <span v-if="parameter.type === 'Immediate'">
                {{ parameter.value }}
              </span>

              <span v-if="parameter.type === 'Offset'">
                {{ parameter.value.offset }}(<span :class="[registers[parameter.value.register].color]">{{ registers[parameter.value.register].name }}</span>)
              </span>
            </span>
          </div>

          <div class="flex items-center mt-4 border-t border-gray-700 p-2">
            <div v-for="register in registerParameters">
              <RegisterItem
                :name="registers[register].name"
                :value="consoleData.registers?.line[register]"
                :classes="registers[register].color"
                :editable="true"
                @set="value => setRegister(register, value)"
              />
            </div>
          </div>
        </div>
      </div>

      <div class="px-2 py-4" v-if="state.stack">
        <span class="text-lg font-semibold">
          Stack
        </span>

        <div class="flex items-center text-neutral-500 border-b border-gray-700 p-2">
          <div class="w-10">

          </div>

          <div class="w-32">
            Address
          </div>

          <div class="w-32">
            Value
          </div>
        </div>

        <div v-for="(value, index) in state.stack.elements" class="flex items-center font-mono my-1">
          <div class="w-10 text-xs text-purple-300">
            {{ value.address === state.stack.sp ? '$sp' : '' }}
          </div>

          <div class="w-32 px-2 py-1 text-neutral-400 hover:bg-neutral-800 rounded">
            0x{{ value.address.toString(16).padStart(8, '0') }}:
          </div>

          <div class="w-32 px-2 py-1 select-all hover:bg-neutral-800 rounded">
            0x{{ value.value.toString(16).padStart(8, '0') }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { consoleData } from '../../state/console-data'
import { computed, onMounted, reactive, watch } from 'vue'
import {
  Breakpoints,
  decodeInstruction,
  ExecutionState,
  InstructionDetails,
  ParameterItemRegular
} from '../../utils/mips'
import { setRegister } from '../../utils/debug'
import RegisterItem from './RegisterItem.vue'

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
  stack: null as StackInfo | null,
  instructions: null as CurrentInstructions | null
})

const registerParameters = computed(() => {
  if (!state.instructions) {
    return []
  }

  // Not worries about optimization here, we want unique + sorted.
  return [...new Set(state.instructions.instructions
    .flatMap(x => x?.parameters ?? [])
    .filter((x): x is ParameterItemRegular => x.type === 'Register')
    .map(x => x.value))]
    .sort()
})

interface InstructionGroup {
  start: number
  count: number
  index: number
}

function instructionGroup(breakpoints: Breakpoints | null, pc: number): InstructionGroup {
  const failed = { start: pc, count: 1, index: 0 }

  if (!breakpoints) {
    return failed
  }

  const group = breakpoints.pcToGroup.get(pc)

  if (!group || group.pcs.length <= 0) {
    return failed
  }

  const start = group.pcs[0]

  // Assert group.pcs are in increasing order
  return {
    start,
    count: group.pcs.length,
    index: (pc - start) / 4,
  }
}

async function instructionAtAddress(state: ExecutionState, address: number): Promise<InstructionDetails | null> {
  const data = await state.memoryAt(address, 4)

  if (data === null) {
    return null
  }

  return await decodeInstruction(address, combine(data))
}

interface CurrentInstructions {
  instructions: (InstructionDetails | null)[]
  currentIndex: number
}

async function currentInstructions(): Promise<CurrentInstructions | null> {
  if (!consoleData.registers || !consoleData.execution) {
    return null
  }

  const state = consoleData.execution

  const pc = consoleData.registers.pc
  const group = instructionGroup(consoleData.execution.breakpoints, pc)

  const instructions = (await Promise.all(
    [...Array(group.count).keys()]
      .map(index => instructionAtAddress(state, group.start + 4 * index))
  ))

  return {
    instructions,
    currentIndex: group.index
  }
}

function combine(data: (number | null)[], index: number = 0): number {
  return (data[index] ?? 0)
    + (data[index + 1] ?? 0) * 256
    + (data[index + 2] ?? 0) * 256 * 256
    + (data[index + 3] ?? 0) * 256 * 256 * 256
}

interface StackInfo {
  sp: number
  elements: StackElement[]
}

async function stackInfo(): Promise<StackInfo | null> {
  if (!consoleData.registers || !consoleData.execution) {
    return null
  }

  const back = 2
  const items = 12

  const sp = consoleData.registers.line[29]
  const start = Math.max(0, sp - back * 4)

  const data = await consoleData.execution.memoryAt(start, 4 * items)

  if (!data) {
    return null
  }

  const elements = [] as StackElement[]

  for (let index = 0; index < data.length; index += 4) {
    // Avoid << because we don't want signed integers
    const value = combine(data, index)

    elements.push({ address: start + index, value })
  }

  return { sp, elements }
}

async function loadDetails() {
  state.instructions = await currentInstructions()
  state.stack = await stackInfo()
}

watch(
  () => [consoleData.registers, consoleData.execution],
  loadDetails
)

onMounted(loadDetails)
</script>
