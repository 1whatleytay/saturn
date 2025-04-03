const storage = navigator.storage?.getDirectory()

globalThis.onmessage = async (event) => {
  if (!storage) {
    return
  }

  let { path, content } = event.data as { path: string; content: Uint8Array }

  let astorage = await storage
  if (path.startsWith('tmp://')) {
    path = path.slice(6)
    astorage = await astorage.getDirectoryHandle('.tmp', { create: true })
  }
  const file = await astorage.getFileHandle(path, { create: true })
  const writable = await file.createSyncAccessHandle()
  writable.truncate(content.length)
  writable.write(content, { at: 0 })
  writable.close()
}

export {}
