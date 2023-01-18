import { listen } from '@tauri-apps/api/event'

import { editor, createTab, closeTab, loadElf } from '../state/tabs-state'
import { resume, step, pause, stop, build } from "./editor-debug";
import { openInputFile, openElf } from './select-file'
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

    const { name, data } = result

    switch (typeof data) {
      case 'string':
        const split = data.split('\n')

        createTab(name, split.length ? split : [''])
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
