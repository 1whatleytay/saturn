import { MipsBackend } from '../utils/mips/mips'
import { TauriBackend } from '../utils/mips/tauri-backend'
import { WasmBackend } from '../utils/mips/wasm-backend'
import { MidiNote, playNote } from '../utils/midi'
import { ConsoleType, pushConsole } from './console-data'

function createBackend(): MipsBackend {
  if (window.__TAURI__) {
    return new TauriBackend()
  } else {
    return new WasmBackend()
  }
}

export const backend = createBackend()

export function setupBackend(): Promise<void> {
  return backend.setCallbacks({
    consoleWrite(text: string, error: boolean) {
      pushConsole(
        text,
        error ? ConsoleType.Stderr : ConsoleType.Stdout
      )
    },

    async midiPlay(note: MidiNote) {
      await playNote(note)
    }
  })
}
