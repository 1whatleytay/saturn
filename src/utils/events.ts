import { listen } from '@tauri-apps/api/event'

import { editor, createTab, closeTab, loadElf, tab, collectLines } from '../state/tabs-state'
import { resume, step, pause, stop, build } from "./editor-debug";
import { openInputFile, openElf, selectSaveAssembly, writeFile, readInputFile, SelectedFile } from './select-file'
import { consoleData } from '../state/console-data'

export enum PromptType {
  NeverPrompt,
  PromptWhenNeeded,
  ForcePrompt
}

export async function openTab(file: SelectedFile<string | Uint8Array>) {
  const { name, path, data } = file

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
}

export async function saveCurrentTab(type: PromptType = PromptType.PromptWhenNeeded) {
  const current = tab()

  if (!current) {
    return
  }

  if (type === PromptType.NeverPrompt && !current.path) {
    return
  }

  if (type === PromptType.ForcePrompt || !current.path) {
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
}

export async function setupEvents() {
  await listen('new-tab', () => {
    createTab('Untitled', [''])
  })

  await listen('open-file', async () => {
    const result = await openInputFile()

    if (!result) {
      return
    }

    await openTab(result)
  })

  await listen('close-tab', () => {
    if (editor.selected) {
      closeTab(editor.selected)
    }
  })

  await listen('save', async () => {
    await saveCurrentTab()
  })

  await listen('save-as', async () => {
    await saveCurrentTab(PromptType.ForcePrompt)
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

  await listen('tauri://file-drop', async event => {
    for (const item of event.payload as string[]) {
      const file = await readInputFile(item)

      if (!file) {
        continue
      }

      await openTab(file)
    }
  })
}
