import { createRouter, createWebHashHistory } from 'vue-router'

import GettingStarted from './snippets/GettingStarted.vue'
import Interpreter from './snippets/Interpreter.vue'

const routes = [
  { path: '/', component: GettingStarted },
  { path: '/interpret', component: Interpreter }
]

export default createRouter({
  history: createWebHashHistory(),
  routes
})