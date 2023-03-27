import { appWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api'
import { hasActionKey } from './query/shortcut-key'

export function setupWindow() {
  window.addEventListener('keydown', event => {
    // Prevent the Ctrl + R refresh.
    if (hasActionKey(event) && event.key === 'r') {
      event.preventDefault()
    }
  })

  const handler = (event: Event) => {
    // Don't bring up the "reload" context menu. It's note great!
    event.preventDefault()
  }

  window.addEventListener('contextmenu', handler)

  // Why this convoluted?
  //  - I want to have the context menu available on --debug and vite builds.
  //  - import.meta.env.DEV is false on --debug builds.
  //  - #[cfg(debug_assertions)] seems like the best way find if the debug flag is set.
  invoke('is_debug')
    .then(result => {
      if (result) {
        window.removeEventListener('contextmenu', handler)
      }
    })
}

// Restricting tauri calls to certain files.
export async function closeWindow() {
  await appWindow.close()
}
