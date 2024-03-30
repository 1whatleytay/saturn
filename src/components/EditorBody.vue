<template>
  <div
    ref="code"
    class="font-mono text-sm flex-auto flex-grow overflow-auto flex pt-2"
  >

      <Suggestions />
      <GotoOverlay
        v-if="gotoHighlights.state.highlight"
        @click="jumpGoto"
        :highlight="gotoHighlights.state.highlight"
      />
      <ErrorOverlay
        v-if="errorHighlights.state.highlight"
        :highlight="errorHighlights.state.highlight"
      />
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'

import {
  goto,
  gotoHighlights,
  errorHighlights,
  tab,
} from '../state/state'

import GotoOverlay from './GotoOverlay.vue'
import ErrorOverlay from './ErrorOverlay.vue'
import Suggestions from './Suggestions.vue'

import { EditorView} from "codemirror"

const code = ref(null as HTMLElement | null)

onMounted(() => {
  const view = new EditorView({
    state: tab()?.state,
    parent: code.value!,
  })

  watch(
    () => tab()?.state,
    (state) => {
      if (state != view.state) {
        console.log(state, view.state)
        view.setState(state!)
      }
    }
  )
})


function jumpGoto() {
  const index = goto.jump()

  tab()?.state

  // if (index && cursor) {
  //   cursor.line = index.line
  //   cursor.index = index.index

  //   cursor.highlight = null
  // }
}

</script>
