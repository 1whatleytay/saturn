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

  // https://gist.github.com/shimondoodkin/1081133
  if (/AppleWebKit\/([\d.]+)/.exec(navigator.userAgent)) {
    view.contentDOM.addEventListener('blur', (): void => {
      var editableFix = document.createElement('input')
      editableFix.style = 'width:1px;height:1px;border:none;margin:0;padding:0;'
      editableFix.tabIndex = -1
      view.contentDOM.appendChild(editableFix)
      editableFix.focus();
      editableFix.setSelectionRange(0, 0);
      editableFix.blur();
      editableFix.remove();
    }, false);
  }

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
