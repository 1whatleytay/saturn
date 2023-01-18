import { createApp } from 'vue'

import './style.css'

import App from './App.vue'
import { setupEvents } from './utils/events'

createApp(App)
  .mount('#app')

setupEvents()
  .then(() => {})
