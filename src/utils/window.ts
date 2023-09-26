import { appWindow } from '@tauri-apps/api/window'
import { hasActionKey } from './query/shortcut-key'

export function setupWindow() {
  window.addEventListener('keydown', (event) => {
    // Prevent the Ctrl + R refresh.
    if (hasActionKey(event) && event.key === 'r') {
      event.preventDefault()
    }
  })

  // const handler = (event: Event) => {
  //   // Don't bring up the "reload" context menu. It's note great!
  //   event.preventDefault()
  // }
  //
  // if (!import.meta.env.TAURI_DEBUG) {
  //   window.addEventListener('contextmenu', handler)
  // }
}

// Restricting tauri calls to certain files.
export async function closeWindow() {
  await appWindow.close()
}
