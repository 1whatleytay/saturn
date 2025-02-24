import { createApp } from 'vue'

import './style.css'

import App from './App.vue'
import { setupWindow } from './utils/window'
import { setupBackend } from './state/backend'
import {
  setupTauriEvents,
  setupTauriShortcuts,
} from './utils/events/tauri-shortcuts'
import { setupWebShortcuts } from './utils/events/web-shortcuts'

createApp(App).mount('#app')

setupWindow()

if (window.__TAURI_INTERNALS__) {
  // Needs backend tying.
  setupTauriEvents().then(() => setupTauriShortcuts())
} else {
  setupWebShortcuts()
}

setupBackend().then(() => {})
