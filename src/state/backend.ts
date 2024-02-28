import { MipsBackend } from '../utils/mips/mips'
import { TauriBackend } from '../utils/mips/tauri-backend'
// import { WasmBackend } from '../utils/mips/wasm-backend'

function createBackend(): MipsBackend {
  // if (window.__TAURI__) {
  return new TauriBackend()
  // } else {
  // return new WasmBackend()
  // }
}

export const backend = createBackend()
