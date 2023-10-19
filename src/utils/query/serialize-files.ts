import { HexRegion } from '../mips'

import { createDir, writeBinaryFile, writeFile } from '@tauri-apps/api/fs'
import { join } from '@tauri-apps/api/path'

export async function writeHexContents(destination: string, body: string) {
  console.log('what the fuck')
  console.log({ body })
  const binary = Uint8Array.from(atob(body), c => c.charCodeAt(0))

  await writeBinaryFile(destination, binary)
}

export async function writeHexRegions(destination: string, regions: HexRegion[]) {
  try {
    await createDir(destination, {
      recursive: true
    })
  } catch { }

  for (const region of regions) {
    const path = await join(destination, `${region.name}.txt`)

    await writeHexContents(path, region.data)
  }
}
