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

      <div class="flex items-center flex-wrap w-full -ml-2">
        <div
          v-for="register of section.values"
          :key="register.name"
          class="w-28 py-1 px-0.5 h-12"
          :class="!consoleData.execution ? 'opacity-50' : ''"
        >
          <div class="text-xs pl-2 group" :class="section.classes">
            {{ register.name }}
          </div>

          <NumberField
            v-if="register.value !== undefined"
            :classes="`
              pl-2 py-1 rounded
              hover:bg-neutral-800 select-all cursor-text
              w-28 bg-transparent
              text-sm
              ${register.marked ? 'text-orange-200' : 'text-gray-100'}`"
            :model-value="register.value"
            @update:model-value="value => setRegister(register.id, value)"
            :checker="checker"
            :editable="!!consoleData.execution"
            :hex="settings.registers.format === RegisterFormat.Hexadecimal"
          />
        </div>
      </div>
    </div>

    <div class="w-full h-12" />

    <div class="
      absolute border border-neutral-800 bottom-0 right-0 bg-neutral-900
      rounded text-neutral-300 text-xs mr-6 mb-6 overflow-hidden
    ">
      <button
        class="px-3 py-1 transition-colors"
        :class="buttonClasses(RegisterFormat.Hexadecimal)"
        @click="() => settings.registers.format = RegisterFormat.Hexadecimal"
      >
        Hex
      </button>

      <button
        class="px-3 py-1 transition-colors"
        :class="buttonClasses(RegisterFormat.Decimal)"
        @click="() => settings.registers.format = RegisterFormat.Decimal"
      >
        Dec
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { consoleData } from '../../state/console-data'
import { settings } from '../../state/state'

import { Cog6ToothIcon, Square3Stack3DIcon } from '@heroicons/vue/24/outline'
import { RegisterFormat } from '../../utils/settings'
import { Registers } from '../../utils/mips'
import NumberField from './NumberField.vue'

function buttonClasses(format: RegisterFormat) {
  const use = settings.registers.format === format

  return {
    'hover:bg-slate-800 bg-neutral-800': use,
    'hover:bg-slate-700': !use
  }
}

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
  id: number
  value?: number
  marked: boolean
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

let lastRegisters = null as Registers | null

watch(() => consoleData.execution, value => {
  if (!value) {
    lastRegisters = null
  }
})

watch(() => consoleData.registers, (value, old) => {
  if (!!old) {
    lastRegisters = old
  }
})

function checker(value: number): string | null {
  const valid = value >= 0 && value <= 0xffffffff

  return !valid ? 'Value must be between 0x0 and 0xffffffff' : null
}

function register(name: string, id: number, get: (registers: Registers) => number | undefined): RegisterValue {
  const lastState = lastRegisters
  const currentState = consoleData.registers

  const value = currentState ? get(currentState) : undefined

  const marked = !!lastState && !!value && get(lastState) !== value

  return { name, id, value, marked }
}

const mappedSections = computed((): RegisterSection[] => {
  const system = [
    register('pc', 34, r => r.pc),
    register('hi', 32, r => r.hi),
    register('lo', 33, r => r.lo),
  ] as RegisterValue[]

  return sections.map(section => {
    const values = section.system ? system.slice() : []

    values.push(...section.indices.map(index =>
      register(registers[index], index, r => r.line[index])
    ))

    return {
      name: section.name,
      classes: section.classes,
      values
    }
  })
})

function setRegister(id: number, value: number) {
  if (consoleData.execution && consoleData.registers) {
    switch (id) {
      case 32:
        consoleData.registers.hi = value
        break

      case 33:
        consoleData.registers.lo = value
        break

      case 34:
        consoleData.registers.pc = value
        break

      default:
        consoleData.registers.line[id] = value
        break
    }
    consoleData.execution.setRegister(id, value)
  }
}
</script>
