import { UAParser } from 'ua-parser-js'

function metaKeyPlatform(): boolean {
  const os = new UAParser(navigator.userAgent).getOS()

  switch (os.name) {
    case 'Mac OS': return true
    default: return false
  }
}

export const isMetaKey = metaKeyPlatform()

interface ActionMarkedEvent {
  metaKey: boolean
  ctrlKey: boolean
}

export function hasActionKey(event: ActionMarkedEvent): boolean {
  return isMetaKey ? event.metaKey : event.ctrlKey
}
