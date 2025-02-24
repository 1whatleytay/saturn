import { emit, listen } from '@tauri-apps/api/event'
import { build, pause, postBuildMessage, resume, step, stop } from '../debug'
import { watch } from 'vue'
import { consoleData, ConsoleType, pushConsole } from '../../state/console-data'
import { closeTab, tab, tabsState } from '../../state/state'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import {
  accessReadFile,
  elfFilter,
} from '../query/access-manager/access-manager-tauri'
import {
  newTab,
  closeCurrentTab,
  assemble,
  disassemble,
  toggleConsole,
  toggleSettings,
  Accelerator,
  exportHex,
  openFile,
  save,
  saveAs,
  openTab,
} from './events'
import { BinaryResult } from '../mips/mips'
import { backend } from '../../state/backend'
import { exportBinaryContents } from '../query/serialize-files'
import { hasActionKey } from '../query/shortcut-key'
import { invoke } from '@tauri-apps/api/core'

interface ConsoleEvent {
  uuid?: string
  message: string
}

// 'export'
export async function exportBinary() {
  const current = tab()

  if (!current) {
    return
  }

  let binary: Uint8Array | null
  let result: BinaryResult | null = null

  if (current.profile && current.profile.kind === 'elf') {
    binary = Uint8Array.from(window.atob(current.profile.elf), (c) =>
      c.charCodeAt(0),
    )
  } else {
    result = await backend.assembleWithBinary(
      current.doc.toString(),
      current.path,
    )

    binary = result.binary
  }

  let destination: string | null = null

  if (binary !== null) {
    destination = await exportBinaryContents(binary.buffer, elfFilter)
  }

  consoleData.showConsole = true

  if (result !== null) {
    postBuildMessage(result.result)
  }

  if (destination !== null) {
    pushConsole(`ELF file written to ${destination}`, ConsoleType.Info)
  }
}

// Why is setupTauriEvents separate from setupTauriShortcuts?
// Because the tauri backend may dispatch these events (that we want to listen for).
// And the frontend only dispatches these events on some platforms (windows).
export async function setupTauriEvents() {
  await listen('new-tab', newTab)
  await listen('open-file', openFile)
  await listen('close-tab', closeCurrentTab)
  await listen('save', save)
  await listen('save-as', saveAs)

  await listen('build', build)
  await listen('run', resume)
  await listen('step', step)
  await listen('pause', pause)
  await listen('stop', stop)

  await listen('assemble', assemble)
  await listen('export', exportBinary)
  await listen('export-hex', exportHex)
  await listen('disassemble', disassemble)

  await listen('toggle-console', toggleConsole)
  await listen('toggle-settings', toggleSettings)

  let events = new Map<string, number>() // uuid to number
  watch(
    () => consoleData.console,
    () => (events = new Map()),
  )

  await listen('post-console-event', (event) => {
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

  await listen('save:create', (event) => {
    const path = event.payload as string

    for (const tab of tabsState.tabs) {
      if (tab.path === path) {
        tab.removed = false
      }
    }
  })

  await listen('save:remove', (event) => {
    const path = event.payload as string

    for (const tab of tabsState.tabs) {
      if (tab.path === path) {
        tab.removed = true
      }
    }
  })

  await listen('save:modify', (event) => {
    const modification = event.payload as {
      path: string
      data: any
    }

    if (typeof modification.data !== 'string') {
      return
    }

    for (const tab of tabsState.tabs) {
      if (tab.path === modification.path) {
        tab.doc = modification.data

        tab.marked = false
      }
    }
  })

  const appWindow = getCurrentWebviewWindow()

  await appWindow.onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop') {
      for (const item of event.payload.paths) {
        const file = await accessReadFile(item)

        if (!file) {
          continue
        }

        await openTab(file)
      }
    }
  })

  await appWindow.onCloseRequested(async (event) => {
    const ids = tabsState.tabs.map((x) => x.uuid)

    for (const id of ids) {
      if (!closeTab(id)) {
        event.preventDefault()
      }
    }
  })
}

interface Shortcut {
  event: string
  accelerator: Accelerator
}

// Tauri Shortcuts - Windows specifically doesn't seem to send Tauri Events for menu shortcuts
// So we have to listen for the events ourselves!
// This is a tauri only method
export async function setupTauriShortcuts() {
  const shortcuts = (await invoke('platform_shortcuts')) as Shortcut[]

  if (!shortcuts.length) {
    return
  }

  const keys = new Map<string, Shortcut[]>()

  shortcuts.forEach((shortcut) => {
    if (!shortcut.accelerator) {
      return
    }

    let list = keys.get(shortcut.accelerator.key)

    if (!list) {
      list = []
      keys.set(shortcut.accelerator.key, list)
    }

    list.push(shortcut)
  })

  window.addEventListener('keydown', async (event) => {
    const shortcut = keys.get(event.key.toUpperCase())?.find((shortcut) => {
      return (
        shortcut.accelerator.command == hasActionKey(event) &&
        shortcut.accelerator.shift == event.shiftKey
      )
    })

    if (!shortcut) {
      return
    }

    event.preventDefault()
    await emit(shortcut.event)
  })
}
