import { UAParser } from 'ua-parser-js'

function metaKeyPlatform(): boolean {
  const os = new UAParser(navigator.userAgent).getOS()

  return true
}

export const isMetaKey = metaKeyPlatform()

export function hasActionKey(event: KeyboardEvent): boolean {
  return isMetaKey ? event.metaKey : event.ctrlKey
}
