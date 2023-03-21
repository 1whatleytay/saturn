import { listen } from '@tauri-apps/api/event'

import { collectLines, EditorTab } from './tabs'
import { resume, step, pause, stop, build, postBuildMessage } from './debug'
import { openInputFile, openElf, selectSaveAssembly, writeFile, readInputFile, SelectedFile } from './query/select-file'
import { consoleData, ConsoleType, pushConsole } from '../state/console-data'
import { assembleWithBinary } from './mips'
import { find, suggestions, tabsState, createTab, closeTab, loadElf, tab } from '../state/state'
import { appWindow } from '@tauri-apps/api/window'
import { watch } from 'vue'
import { MidiNote, playNote } from './midi'
import { splitLines } from './split-lines'

export enum PromptType {
  NeverPrompt,
  PromptWhenNeeded,
  ForcePrompt
}

interface ConsoleEvent {
  uuid?: string
  message: string
}

export async function openTab(file: SelectedFile<string | Uint8Array>) {
  const { name, path, data } = file

  const existing = tabsState.tabs.find(tab => tab.path === path)

  if (existing) {
    tabsState.selected = existing.uuid
    return
  }

  switch (typeof data) {
    case 'string':
      const split = splitLines(data)

      createTab(name, split.length ? split : [''], path)
      break

    default:
      await loadElf(name, data.buffer)
      break
  }
}

export async function saveTab(current: EditorTab, type: PromptType = PromptType.PromptWhenNeeded): Promise<boolean> {
  if (type === PromptType.NeverPrompt && !current.path) {
    return true
  }

  if (type === PromptType.ForcePrompt || !current.path) {
    const result = await selectSaveAssembly()

    if (!result) {
      return false
    }

    const { name, path } = result

    current.title = name
    current.path = path
  }

  const data = collectLines(current.lines)

  await writeFile(current.path, data)

  current.marked = false // Remove "needs saving" marker

  return true
}

export async function saveCurrentTab(prompt: PromptType = PromptType.PromptWhenNeeded) {
  const current = tab()

  if (current) {
    await saveTab(current, prompt)
  }
}

interface PrintPayload {
  text: string,
  error: boolean
}

export async function setupEvents() {
  await listen('print', event => {
    let payload = event.payload as PrintPayload

    pushConsole(payload.text, payload.error ? ConsoleType.Stderr : ConsoleType.Stdout)
  })

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
    if (tabsState.selected) {
      closeTab(tabsState.selected)
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

  await listen('find', () => {
    find.state.show = true
    find.state.focus = true // send focus event
    suggestions.dismissSuggestions()
  })

  await listen('assemble', async () => {
    const result = await assembleWithBinary(collectLines(tab()?.lines ?? []))

    if (result.binary) {
      const name = tab()?.title
      const extended = name ? `${name}.elf` : 'Untitled Elf'

      await loadElf(extended, result.binary)
    }

    consoleData.showConsole = true
    postBuildMessage(result.result)
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

  await listen('play-midi', async event => {
    await playNote(event.payload as MidiNote)
  })

  let events = new Map<string, number>() // uuid to number
  watch(() => consoleData.console, () => events = new Map())

  await listen('post-console-event', event => {
    const payload = event.payload as ConsoleEvent

    const push = () => pushConsole(payload.message, ConsoleType.Info)

    if (payload.uuid) {
      const id = events.get(payload.uuid)

      if (id) {
        consoleData.console[id] = payload.message
      } else {
        events.set(payload.uuid, push())
      }
    } else {
      push()
    }
  })

  await appWindow.onFileDropEvent(async event => {
    if (event.payload.type === 'drop') {
      for (const item of event.payload.paths) {
        const file = await readInputFile(item)

        if (!file) {
          continue
        }

        await openTab(file)
      }
    }
  })

  await appWindow.onCloseRequested(async event => {
    const ids = tabsState.tabs.map(x => x.uuid)

    for (const id of ids) {
      if (!closeTab(id)) {
        event.preventDefault()
      }
    }
  })
}
