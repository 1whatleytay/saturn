import { consoleData, ConsoleType, DebugTab, openConsole, pushConsole } from '../state/console-data'

import * as wasm from './wasm/asm'

export function interact() {
  consoleData.showConsole = true
  consoleData.tab = DebugTab.Console

  let x = wasm.greet('v: b v')

  openConsole()

  pushConsole(`Greet complete: ${x.length}`, ConsoleType.Info)
  pushConsole(`Greet complete: ${x.toString()}`, ConsoleType.Info)
}
