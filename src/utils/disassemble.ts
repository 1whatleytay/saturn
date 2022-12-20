import { tauri } from '@tauri-apps/api'

export async function disassembleElf(elf: ArrayBuffer): Promise<string[]> {
  const bytes = Array.from(new Uint8Array(elf))

  return tauri.invoke('disassemble', { bytes })
}