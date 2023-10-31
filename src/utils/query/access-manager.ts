import { invoke } from '@tauri-apps/api'

export interface AccessFile<T> {
  path: string
  name: string | null
  extension: string | null
  data: T
}

export interface AccessFilter {
  name: string,
  extensions: string[]
}

export const assemblyFilter: AccessFilter[] = [
  {
    name: 'Assembly',
    extensions: ['asm', 's'],
  },
]

export const elfFilter: AccessFilter[] = [
  {
    name: 'ELF',
    extensions: ['elf'],
  },
]

export async function selectSaveDestination(
  title: string, filters?: AccessFilter[]
): Promise<AccessFile<undefined> | null> {
  return await invoke('access_select_save', {
    title, filters: filters ?? []
  }) as AccessFile<undefined> | null
}

export async function selectOpenElf(): Promise<AccessFile<Uint8Array> | null> {
  return await invoke('access_select_open', {
    title: 'Select ELF', filters: elfFilter, selection: 'all_binary'
  })
}

export async function selectOpenFile(
  title: string, filters?: AccessFilter[]
): Promise<AccessFile<string | Uint8Array> | null> {
  return await invoke('access_select_open', {
    title, filters: filters ?? [], selection: { binary: ['elf'] }
  })
}

export async function accessWriteText(path: string, content: string): Promise<void> {
  return await invoke('access_write_text', { path, content })
}

export async function accessReadText(path: string): Promise<string> {
  return await invoke('access_read_text', { path })
}

export async function accessSync(paths: string[]) {
  await invoke('access_sync', { paths })
}
