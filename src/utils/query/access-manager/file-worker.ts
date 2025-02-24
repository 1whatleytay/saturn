const storage = navigator.storage?.getDirectory()

globalThis.onmessage = async (event) => {
  if (!storage) {
    return
  }

  const { path, content } = event.data
  const astorage = await storage
  const file = await astorage.getFileHandle(path, { create: true })
  const writable = await file.createSyncAccessHandle()
  writable.write(content)
  writable.close()
}
