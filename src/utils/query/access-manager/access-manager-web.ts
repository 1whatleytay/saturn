import { showFileOpenDialog, showFileSaveDialog } from '../../../state/state'
import { AccessFile } from '.'
import FileWorker from './file-worker?worker'

const worker = new FileWorker()
const storage = navigator.storage?.getDirectory()

let showFileSaveResolve: ((t: string) => void) | null = null
export const confirm = (t: string) => {
  showFileSaveResolve?.(t)
}

export const getOpenableFiles = async () => {
  if (!storage) {
    return []
  }

  const files = (await storage).keys()
  const out = []
  for await (const key of files) {
    out.push(key)
  }
  return out
}

export async function selectSaveDestination(): Promise<AccessFile<undefined> | null> {
  showFileSaveDialog.value = true
  const name = await new Promise<string>((resolve) => {
    showFileSaveResolve = resolve
  })
  showFileSaveDialog.value = false

  if (!name) {
    return null
  }

  return {
    path: name,
    name: name,
    extension: '',
    data: undefined,
  }
}

export async function accessWriteText(
  path: string,
  content: string,
): Promise<void> {
  const textEncoder = new TextEncoder()
  const encoded = textEncoder.encode(content)
  await accessWriteBinary(path, encoded)
}

export async function accessWriteBinary(
  path: string,
  content: Uint8Array,
): Promise<void> {
  worker.postMessage({ path, content })
}

export async function selectOpenFile(): Promise<AccessFile<
  string | Uint8Array
> | null> {
  showFileOpenDialog.value = true
  const name = await new Promise<string>((resolve) => {
    showFileSaveResolve = resolve
  })
  showFileOpenDialog.value = false

  return await accessReadFile(name)
}

export async function accessReadText(path: string): Promise<string> {
  const astorage = await storage
  const file = await astorage.getFileHandle(path)
  const fileContents = await file.getFile()
  return await fileContents.text()
}

export async function accessReadFile(
  path: string,
): Promise<AccessFile<string | Uint8Array>> {
  const astorage = await storage
  const file = await astorage.getFileHandle(path)
  const fileContents = await file.getFile()
  const data = fileContents.name.endsWith('.elf')
    ? new Uint8Array(await fileContents.arrayBuffer())
    : await fileContents.text()
  return {
    path,
    name: path,
    extension: path.split('.').pop() ?? '',
    data,
  }
}
