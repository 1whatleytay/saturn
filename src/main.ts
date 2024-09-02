import { createApp } from 'vue'

import './style.css'

import App from './App.vue'
import { setupEvents } from './utils/events'
import { setupShortcuts } from './utils/platform-shortcuts'
import { setupWindow } from './utils/window'

createApp(App).mount('#app')

setupWindow()

if (window.__TAURI__) {
  // Needs backend tying.
  setupShortcuts().then(() => {})
  setupEvents().then(() => {})
}