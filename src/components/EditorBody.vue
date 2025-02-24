<template>
  <div
    ref="code"
    class="font-mono text-sm flex-auto flex-grow overflow-auto flex pt-2 bg-neutral-100 dark:bg-neutral-900"
    :style="{ '--font-size': settings.editor.fontSize + 'px' }"
  ></div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { errorHighlights, tab, settings } from '../state/state'
import { isSyncing } from '../utils/tabs'

import { EditorView } from 'codemirror'
import { clearHighlightedLine } from '../utils/lezer-mips'
import { consoleData } from '../state/console-data'
import { setHighlightedLine } from '../utils/lezer-mips'
import { setMinimap, setVim, setTheme } from '../utils/lezer-mips/modes'
import { Diagnostic, setDiagnostics } from '@codemirror/lint'
import { hostYTab } from '../utils/lezer-mips/collab'

const code = ref(null as HTMLElement | null)

onMounted(() => {
  const view = new EditorView({
    state: undefined,
    parent: code.value!,
  })

  watch(
    () => tab()?.state,
    (state) => {
      if (!isSyncing() && state) {
        view.setState(state!)

        // sync global settings on tab switch
        view.dispatch({
          effects: [
            setTheme(settings.editor.darkMode),
            setVim(settings.editor.vimMode),
            setMinimap(settings.editor.showMinimap),
          ],
        })
      }
    },
    { flush: 'sync', immediate: true },
  )

  watch(
    () => settings.editor.darkMode,
    (theme: boolean) => view.dispatch({ effects: [setTheme(theme)] }),
  )

  watch(
    () => settings.editor.vimMode,
    (vimMode: boolean) => view.dispatch({ effects: [setVim(vimMode)] }),
  )

  watch(
    () => settings.editor.showMinimap,
    (minimap: boolean) => view.dispatch({ effects: [setMinimap(minimap)] }),
  )

  watch(
    () => settings.editor.fontSize,
    () => view.requestMeasure(),
  )

  ;(window as any).host = () => {
    view.dispatch({ effects: [hostYTab(tab()!)] })
  }

  // https://gist.github.com/shimondoodkin/1081133
  if (/AppleWebKit\/([\d.]+)/.exec(navigator.userAgent)) {
    view.contentDOM.addEventListener(
      'blur',
      (e): void => {
        if (!e.relatedTarget) return

        var editableFix = document.createElement('input')
        editableFix.setAttribute(
          'style',
          'width:1px;height:1px;border:none;margin:0;padding:0;',
        )
        document.body.appendChild(editableFix)
        editableFix.setSelectionRange(0, 0)
        editableFix.blur()
        editableFix.remove()
      },
      false,
    )
  }

  watch(
    () => errorHighlights.state.highlight,
    (highlight) => {
      const diagnostics: Diagnostic[] = []
      if (highlight) {
        let lineI = highlight.line + 1
        let line
        do {
          line = view.state.doc.line(lineI)
          lineI++
        } while (/^\s*$/.test(line.text))

        let offset = highlight.offset
        while (/\s/.test(line.text[offset])) offset++
        let end = offset
        while (
          /[a-zA-Z_\-0-9$.%]/.test(line.text[end]) &&
          end < line.text.length
        )
          end++

        diagnostics.push({
          from: line.from + offset,
          to: line.from + end,
          message: highlight.message,
          severity: 'error',
        })
      }
      view.dispatch(setDiagnostics(view.state, diagnostics))
    },
  )

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
    } else {
      view.dispatch({ effects: [clearHighlightedLine.of(null)] })
    }
  })
})
</script>
