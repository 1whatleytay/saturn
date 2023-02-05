import { createApp } from 'vue'

import './style.css'

import App from './App.vue'
import { setupEvents } from './utils/events'
import { setupShortcuts } from './utils/platform-shortcuts'

createApp(App)
  .mount('#app')

setupShortcuts()
  .then(() => {})

setupEvents()
  .then(() => {})

