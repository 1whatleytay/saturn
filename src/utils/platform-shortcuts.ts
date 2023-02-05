import { invoke } from '@tauri-apps/api'
import { hasActionKey } from './query/shortcut-key'
import { emit } from '@tauri-apps/api/event'

interface Accelerator {
  command: boolean,
  shift: boolean,
  key: string
}

interface Shortcut {
  event: string,
  accelerator: Accelerator
}

export async function setupShortcuts() {
  const shortcuts: Shortcut[] = await invoke('platform_shortcuts')

  const keys = new Map<string, Shortcut[]>()

  shortcuts.forEach(shortcut => {
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

  window.addEventListener('keydown', async event => {
    const shortcut = keys.get(event.key)?.find(x => {
      return x.accelerator.command == hasActionKey(event)
        && x.accelerator.shift == event.shiftKey
    })

    if (!shortcut) {
      return
    }

    await emit(shortcut.event)
  })
}
