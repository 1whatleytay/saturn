import { UAParser } from 'ua-parser-js'

const os = new UAParser(navigator.userAgent).getOS()

function metaKeyPlatform(): boolean {
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

interface AltMarkedEvent {
  altKey: boolean
  ctrlKey: boolean
}

export function hasActionKey(event: ActionMarkedEvent): boolean {
  return isMetaKey ? event.metaKey : event.ctrlKey
}

export function hasAltKey(event: AltMarkedEvent): boolean {
  return event.altKey || (isMetaKey && event.ctrlKey)
}
