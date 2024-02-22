import { HexRegion } from '../mips'

import { invoke } from '@tauri-apps/api'
import { AccessFilter } from './access-manager'

export async function exportBinaryContents(data: ArrayBuffer, filters: AccessFilter[]): Promise<string | null> {
  try {
    return await invoke('export_binary_contents', {
      data: Array.from(new Uint8Array(data)),
      filters
    })
  } catch {
    return null
  }
}

export async function exportHexContents(data: string): Promise<string | null> {
  try {
    return await invoke('export_hex_contents', { data })
  } catch {
    return null
  }
}

export async function exportHexRegions(regions: HexRegion[]): Promise<string | null> {
  try {
    return await invoke('export_hex_regions', { regions })
  } catch {
    return null
  }
}
