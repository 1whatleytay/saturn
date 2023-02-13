import { appWindow } from '@tauri-apps/api/window'

// Restricting tauri calls to certain files.
export async function closeWindow() {
  await appWindow.close()
}
