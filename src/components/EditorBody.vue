<template>
  <div
    ref="code"
    class="font-mono text-sm flex-auto flex-grow overflow-auto flex pt-2 bg-neutral-200 dark:bg-neutral-900"
  >
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
import { computed, onMounted, ref, watch } from 'vue'

import {
  goto,
  gotoHighlights,
  errorHighlights,
  tab,
  settings,
} from '../state/state'

import GotoOverlay from './GotoOverlay.vue'
import ErrorOverlay from './ErrorOverlay.vue'

import { EditorView } from 'codemirror'
import { darkTheme, editorTheme, lightTheme } from '../utils/lezer-mips'
import { consoleData } from '../state/console-data'
import { setHighlightedLine } from '../utils/lezer-mips'
import { setMinimap, setVim } from '../utils/lezer-mips/modes'
import {Diagnostic, setDiagnostics} from '@codemirror/lint'

const code = ref(null as HTMLElement | null)

onMounted(() => {
  const view = new EditorView({
    state: tab()?.state,
    parent: code.value!,
  })

  watch(
    () => settings.editor.darkMode,
    (theme: boolean) => {
      // A strange bug in codemirror made launching in the opposite theme fail without a timeout.
      setTimeout(() => {
        view.dispatch({
          effects: [editorTheme.reconfigure(theme ? darkTheme : lightTheme)],
        })
      }, 0)
    },
    { immediate: true }
  )

  watch(
    () => settings.editor.vimMode,
    (vimMode: boolean) => view.dispatch({ effects: [setVim(vimMode)] }),
    { immediate: true }
  )

  watch(
    () => settings.editor.showMinimap,
    (minimap: boolean) => view.dispatch({ effects: [setMinimap(minimap)] }),
  )

  // https://gist.github.com/shimondoodkin/1081133
  if (/AppleWebKit\/([\d.]+)/.exec(navigator.userAgent)) {
    view.contentDOM.addEventListener(
      'blur',
      (): void => {
        var editableFix = document.createElement('input')
        editableFix.style =
          'width:1px;height:1px;border:none;margin:0;padding:0;'
        editableFix.tabIndex = -1
        view.contentDOM.appendChild(editableFix)
        editableFix.focus()
        editableFix.setSelectionRange(0, 0)
        editableFix.blur()
        editableFix.remove()
      },
      false,
    )
  }

  watch(
    () => tab()?.state,
    (state) => {
      if (state != view.state) {
        console.log(state, view.state)
        view.setState(state!)
      }
    },
  )

  watch(() => errorHighlights.state.highlight, (highlight) => {
    const diagnostics: Diagnostic[] = []
    if (highlight) {
      let lineI = highlight.line;
      let line;
      do {
        line = view.state.doc.line(lineI);
        lineI++
      } while (/^\s*$/.test(line.text));

      let offset = highlight.offset;
      while (/\s/.test(line.text[offset])) offset++;
      let end = offset;
      while (/[a-zA-Z_\-0-9$.%]/.test(line.text[end]) && end < line.text.length) end++;

      diagnostics.push({
        from: line.from + offset,
        to: line.from + end,
        message: highlight.message,
        severity: 'error'
      })
    }
    view.dispatch(setDiagnostics(view.state, diagnostics))
  })

  const stoppedIndex = computed(() => {
    const profile = tab()?.profile
    const registers = consoleData.registers
    const execution = consoleData.execution

    if (!profile || !registers || !execution) {
      return null
    }

    // Reactivity concern here (eh... not too bad, we just want to listen to changes in debug).
    let point = execution.breakpoints?.pcToGroup.get(registers.pc)?.line

    if (point === undefined && consoleData.hintPc != null) {
      point = execution.breakpoints?.pcToGroup.get(consoleData.hintPc)?.line
    }

    return point ?? null
  })

  watch(stoppedIndex, (index) => {
    if (index !== null) {
      const pos = view.state.doc.line(index + 1).from
      view.dispatch({
        effects: [setHighlightedLine.of(pos), EditorView.scrollIntoView(pos)],
      })
    }
  })
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
