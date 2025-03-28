const storage = navigator.storage?.getDirectory()

globalThis.onmessage = async (event) => {
  if (!storage) {
    return
  }

  const { path, content } = event.data as { path: string; content: Uint8Array }
  const astorage = await storage
  const file = await astorage.getFileHandle(path, { create: true })
  const writable = await file.createSyncAccessHandle()
  writable.truncate(content.length)
  writable.write(content, { at: 0 })
  writable.close()
}

export {}
