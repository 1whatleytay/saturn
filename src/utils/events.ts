import { listen } from '@tauri-apps/api/event'

import { editor, createTab, closeTab, loadElf, tab, collectLines } from '../state/tabs-state'
import { resume, step, pause, stop, build } from "./editor-debug";
import { openInputFile, openElf, selectSaveAssembly, writeFile } from './select-file'
import { consoleData } from '../state/console-data'

export async function setupEvents() {
  await listen('new-tab', () => {
    createTab('Untitled', [''])
  })

  await listen('open-file', async () => {
    const result = await openInputFile()

    if (!result) {
      return
    }

    const { name, path, data } = result

    const existing = editor.tabs.find(tab => tab.path === path)

    if (existing) {
      editor.selected = existing.uuid
      return
    }

    switch (typeof data) {
      case 'string':
        const split = data.split('\n')

        createTab(name, split.length ? split : [''], path)
        break

      default:
        await loadElf(name, data.buffer)
        break
    }
  })

  await listen('close-tab', () => {
    if (editor.selected) {
      closeTab(editor.selected)
    }
  })

  await listen('save', async () => {
    const current = tab()

    if (!current) {
      return
    }

    if (!current.path) {
      const result = await selectSaveAssembly()

      if (!result) {
        return
      }

      const { name, path } = result

      current.title = name
      current.path = path
    }

    const data = collectLines(current.lines)

    await writeFile(current.path, data)

    current.marked = false // Remove "needs saving" marker
  })

  await listen('build', async () => {
    await build()
  })

  await listen('run', async () => {
    await resume()
  })

  await listen('step', async () => {
    await step()
  })

  await listen('pause', async () => {
    await pause()
  })

  await listen('stop', async () => {
    await stop()
  })

  await listen('disassemble', async () => {
    const result = await openElf()

    if (!result) {
      return
    }

    const { name, data } = result

    await loadElf(name, data.buffer)
  })

  await listen('toggle-console', () => {
    consoleData.showConsole = !consoleData.showConsole
  })
}
