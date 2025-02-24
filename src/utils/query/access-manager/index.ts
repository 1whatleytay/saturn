import {
  selectSaveDestination as selectSaveDestinationTauri,
  accessWriteText as accessWriteTextTauri,
  selectOpenFile as selectOpenFileTauri,
  accessReadText as accessReadTextTauri,
  accessReadFile as accessReadFileTauri,
  accessSync as accessSyncTauri,
} from './access-manager-tauri'
import {
  selectSaveDestination as selectSaveDestinationWeb,
  accessWriteText as accessWriteTextWeb,
  selectOpenFile as selectOpenFileWeb,
  accessReadText as accessReadTextWeb,
  accessReadFile as accessReadFileWeb,
} from './access-manager-web'

export interface AccessFile<T> {
  path: string
  name: string | null
  extension: string | null
  data: T
}

export interface AccessFilter {
  name: string
  extensions: string[]
}

export async function selectSaveDestination(
  title: string,
  filters?: AccessFilter[],
): Promise<AccessFile<undefined> | null> {
  if (window.__TAURI_INTERNALS__) {
    return selectSaveDestinationTauri(title, filters)
  } else {
    return selectSaveDestinationWeb()
  }
}

export async function accessWriteText(
  path: string,
  content: string,
): Promise<void> {
  if (window.__TAURI_INTERNALS__) {
    return accessWriteTextTauri(path, content)
  } else {
    return accessWriteTextWeb(path, content)
  }
}

export async function selectOpenFile(
  title: string,
  filters?: AccessFilter[],
): Promise<AccessFile<string | Uint8Array> | null> {
  if (window.__TAURI_INTERNALS__) {
    return selectOpenFileTauri(title, filters)
  } else {
    return selectOpenFileWeb()
  }
}

export async function accessReadText(path: string): Promise<string> {
  if (window.__TAURI_INTERNALS__) {
    return accessReadTextTauri(path)
  } else {
    return accessReadTextWeb(path)
  }
}

export async function accessReadFile(
  path: string,
): Promise<AccessFile<string | Uint8Array>> {
  if (window.__TAURI_INTERNALS__) {
    return accessReadFileTauri(path)
  } else {
    return accessReadFileWeb(path)
  }
}

export async function accessSync(paths: string[]) {
  if (window.__TAURI_INTERNALS__) {
    return await accessSyncTauri(paths)
  }
}
