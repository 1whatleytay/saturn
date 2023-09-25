import { HexRegion } from '../mips'

import { createDir, writeFile } from '@tauri-apps/api/fs'
import { join } from '@tauri-apps/api/path'

export async function writeHexRegions(destination: string, regions: HexRegion[]) {
  try {
    await createDir(destination, {
      recursive: true
    })
  } catch { }

  for (const region of regions) {
    const path = await join(destination, `${region.name}.txt`)

    await writeFile(path, region.data)
  }
}
