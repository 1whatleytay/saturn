<template>
  <div @click.stop>
    <SaveModal :dialog="saveModal" />
    <SettingsModal :show="showSettings" @close="showSettings = false" />
    <ExportOverlay :show="showExportRegionsDialog" @close="showExportRegionsDialog = false" />

    <div
      class="h-10 flex items-start bg-neutral-900 w-full fixed z-20 top-0"
    >
      <div class="flex flex-grow overflow-x-auto no-scrollbar items-start">
        <Tab
          v-for="tab in tabsState.tabs"
          :ref="(component) => refTab(tab.uuid, component)"
          :key="tab.uuid"
          :title="tab.title"
          :marked="tab.marked"
          :removed="tab.removed"
          :selected="tabsState.selected === tab.uuid"
          :deletable="true"
          :style="styleForTab(tab.uuid)"
          @mousedown="(e: MouseEvent) => handleDown(e, tab.uuid)"
          @delete="closeTab(tab.uuid)"
        />

        <button
          class="w-10 h-10 hover:bg-slate-800 text-slate-300 shrink-0 flex items-center justify-center font-black"
          @click="create"
        >
          <PlusIcon class="w-4 h-4" />
        </button>
      </div>

      <TabBarItems class="ml-auto shrink-0" />
    </div>

    <div class="w-full h-10 border-b-2 opacity-0" />
  </div>
</template>

<script setup lang="ts">
import Tab from './Tab.vue'
import { PlusIcon } from '@heroicons/vue/24/solid'

import { closeTab, createTab, saveModal, tabsState, showSettings, showExportRegionsDialog } from '../state/state'

import TabBarItems from './TabBarItems.vue'
import SaveModal from './SaveModal.vue'
import { nextTick, onMounted, onUnmounted, reactive, StyleValue } from 'vue'
import SettingsModal from './SettingsModal.vue'
import ExportOverlay from './ExportModal.vue'

const state = reactive({
  dragging: false,
  start: 0,
  offset: 0,
})

let tabElements = new Map<string, HTMLElement>()

function refTab(uuid: string, component: any) {
  if (component) {
    tabElements.set(uuid, component.$el)
  } else {
    tabElements.delete(uuid)
  }
}

const transitionTransform = {
  transitionProperty: 'transform',
  transitionTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
  transitionDuration: '150ms',
}

function styleForTab(uuid: string): StyleValue {
  if (state.dragging && uuid === tabsState.selected) {
    return {
      transform: `translateX(${state.offset}px)`,
    }
  }

  return transitionTransform
}

function handleDown(event: MouseEvent, uuid: string) {
  tabsState.selected = uuid

  const item = tabElements.get(tabsState.selected)
  if (item) {
    state.dragging = true

    state.start = event.clientX - item.offsetLeft
  }
}

let lastX = 0

const handleMove = async (event: MouseEvent) => {
  const previousX = lastX
  lastX = event.clientX

  if (previousX === event.clientX || !state.dragging || !tabsState.selected) {
    return
  }

  const item = tabElements.get(tabsState.selected)

  if (!item) {
    return
  }

  const rightOnly = previousX < event.clientX

  const offset = event.clientX - state.start - item.offsetLeft
  const middle = item.offsetWidth / 2 + item.offsetLeft + offset

  state.offset = offset

  for (const [key, element] of tabElements.entries()) {
    if (key === tabsState.selected) {
      continue
    }

    const start = element.offsetLeft
    const end = start + element.offsetWidth

    if (start <= middle && middle <= end) {
      const first = tabsState.tabs.findIndex(
        (x) => x.uuid === tabsState.selected
      )
      const second = tabsState.tabs.findIndex((x) => x.uuid === key)

      if (first < 0 || second < 0) {
        continue
      }

      // We don't want to make a lot of swaps in the same place
      if (first < second !== rightOnly) {
        continue
      }

      const temp = tabsState.tabs[first]
      tabsState.tabs[first] = tabsState.tabs[second]
      tabsState.tabs[second] = temp

      await nextTick()
      state.offset = event.clientX - state.start - item.offsetLeft

      break
    }
  }
}

const handleUp = () => {
  state.dragging = false
  state.start = 0
  state.offset = 0
}

onMounted(() => {
  window.addEventListener('mouseup', handleUp)
  window.addEventListener('mousemove', handleMove)
})

onUnmounted(() => {
  window.removeEventListener('mouseup', handleUp)
  window.removeEventListener('mousemove', handleMove)
})

function create() {
  createTab('Untitled', '')
}
</script>
