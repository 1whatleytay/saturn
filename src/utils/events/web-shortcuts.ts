import { hasActionKey } from '../query/shortcut-key'
import {
  Accelerator,
  closeCurrentTab,
  newTab,
  openFile,
  saveAs,
  saveCurrentTab,
  toggleConsole,
  toggleSettings,
} from './events'
import { build, pause, postBuildMessage, resume, step, stop } from '../debug'
import { tab } from '../../state/state'
import {
  accessWriteBinary,
  selectSaveDestination,
} from '../query/access-manager/access-manager-web'
import { consoleData, ConsoleType, pushConsole } from '../../state/console-data'
import { backend } from '../../state/backend'
import { BinaryResult } from '../mips/mips'

// 'export'
export async function exportBinary() {
  const current = tab()

  if (!current) {
    return
  }

  let binary: Uint8Array | null
  let result: BinaryResult | null = null

  if (current.profile && current.profile.kind === 'elf') {
    binary = Uint8Array.from(window.atob(current.profile.elf), (c) =>
      c.charCodeAt(0),
    )
  } else {
    result = await backend.assembleWithBinary(
      current.doc.toString(),
      current.path,
    )

    binary = result.binary
  }

  let destination: string | null = null

  if (binary !== null) {
    const dest = await selectSaveDestination()
    if (!dest) {
      return
    }
    await accessWriteBinary(dest.path, binary)
  }

  consoleData.showConsole = true

  if (result !== null) {
    postBuildMessage(result.result)
  }

  if (destination !== null) {
    pushConsole(`ELF file written to ${destination}`, ConsoleType.Info)
  }
}

function command(key: string, shift?: boolean): Accelerator {
  return {
    command: true, // isn't command always true?
    shift: shift ?? false,
    key,
  }
}

// cmd + N, cmd + T, cmd + W, cmd + L are not preventDefault'able
// do we want to rebind?
const bindings: [Accelerator, () => void][] = [
  [command('N'), newTab],
  [command('O'), openFile],
  [command('W'), closeCurrentTab],
  [command('S'), saveCurrentTab],
  [command('S', true), saveAs], // tauri required
  // [command('F'), () => {}], // not listened to, handled by editor
  [command('B'), build],
  [command('K'), resume],
  [command('L'), step],
  [command('J'), pause],
  [command('P'), stop],
  [command('O'), toggleConsole],
  [command(','), toggleSettings],
]

export function setupWebShortcuts() {
  window.addEventListener('keydown', async (event) => {
    const key = event.key.toUpperCase()

    const shortcut = bindings.find(([accelerator, _]) => {
      return (
        accelerator.key === key &&
        accelerator.command == hasActionKey(event) &&
        accelerator.shift == event.shiftKey
      )
    })

    if (!shortcut) {
      return
    }

    const [_, callback] = shortcut

    event.preventDefault()
    callback()
  })
}
