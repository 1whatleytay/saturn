import { createApp } from 'vue'

import './style.css'

import App from './App.vue'
import { setupEvents } from './utils/events'
import { setupShortcuts } from './utils/platform-shortcuts'
import { setupWindow } from './utils/window'
import { setupBackend } from './state/backend'
import { invoke } from '@tauri-apps/api'

invoke('send_trace', { text: 'wake' })
  .then(() => { })

window.addEventListener('error', async e => {
  await invoke('send_trace', { text: `![JS ERROR]! ${e.message} - (${e.filename}, ${e.lineno}, ${e.colno}) - ${e.error}` })
})

createApp(App).mount('#app')

setupWindow()

if (window.__TAURI__) {
  // Needs backend tying.
  setupEvents().then(() => {})
}

setupBackend().then(backend => {
  setupShortcuts(backend).then(() => {})
})
