import { listen, Event } from '@tauri-apps/api/event'

import { editor, createTab, closeTab } from '../state/tabs-state'
import { resume, step, pause, stop, build } from "./editor-debug";

export async function setupEvents() {
  await listen('new-tab', () => {
    createTab('Untitled', [''])
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
}
