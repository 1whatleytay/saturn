import { UAParser } from 'ua-parser-js'

function metaKeyPlatform(): boolean {
  const os = new UAParser(navigator.userAgent).getOS()

  switch (os.name) {
    case 'Mac OS': return true
    default: return false
  }
}

export const isMetaKey = metaKeyPlatform()

export function hasActionKey(event: KeyboardEvent): boolean {
  return isMetaKey ? event.metaKey : event.ctrlKey
}
