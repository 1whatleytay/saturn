import { open } from '@tauri-apps/api/dialog'
import { basename, extname } from '@tauri-apps/api/path'
import { readBinaryFile, readTextFile } from '@tauri-apps/api/fs'

interface SelectedFile<T> {
  name: string
  data: T
}

export async function openElf(): Promise<SelectedFile<Uint8Array> | null> {
  const result = await open({
    title: 'Select ELF',
    filters: [{
      name: 'ELF',
      extensions: ['elf']
    }],
    multiple: false,
    directory: false
  })

  if (!result || typeof result !== 'string') {
    return null
  }

  const name = await basename(result)
  const data = await readBinaryFile(result)

  return { name, data }
}

// Would be magic if this could also open ELF files.
export async function openInputFile(): Promise<SelectedFile<string | Uint8Array> | null> {
  const result = await open({
    title: 'Select File',
    multiple: false,
    directory: false
  })

  if (!result || typeof result !== 'string') {
    return null
  }

  const name = await basename(result)
  const extension = await extname(result)

  let data: string | Uint8Array

  switch (extension) {
    case 'elf':
      data = await readBinaryFile(result)
      break

    default:
      data = await readTextFile(result)
      break
  }

  return { name, data }
}
