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

import * as Sentry from '@sentry/vue'

const app = createApp(App)

Sentry.init({
  app,
  dsn: 'https://71a1f02b28493812210b0887fe243c06@o4504600183046144.ingest.us.sentry.io/4509175145693184'
})

app.mount('#app')

setupWindow()

if (window.__TAURI_INTERNALS__) {
  // Needs backend tying.
  setupTauriEvents().then(() => setupTauriShortcuts())
} else {
  setupWebShortcuts()
}

setupBackend().then(() => {})
