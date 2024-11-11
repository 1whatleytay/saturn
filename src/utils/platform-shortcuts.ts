import { hasActionKey } from './query/shortcut-key'
import { emit } from '@tauri-apps/api/event'
import { MipsBackend, Shortcut } from './mips/mips'

export async function setupShortcuts(backend: MipsBackend) {
  const shortcuts = await backend.shortcuts()

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
