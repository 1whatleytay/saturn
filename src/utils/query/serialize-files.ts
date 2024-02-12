import { HexRegion } from '../mips'

import { invoke } from '@tauri-apps/api'

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
