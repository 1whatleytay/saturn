// Mac detection taken from https://github.com/codemirror/view/blob/main/src/browser.ts
let nav: any =
  typeof navigator != 'undefined'
    ? navigator
    : { userAgent: '', vendor: '', platform: '' }
const ie_edge = /Edge\/(\d+)/.exec(nav.userAgent)
const ie_upto10 = /MSIE \d/.test(nav.userAgent)
const ie_11up = /Trident\/(?:[7-9]|\d{2,})\..*rv:(\d+)/.exec(nav.userAgent)
const ie = !!(ie_upto10 || ie_11up || ie_edge)
const safari = !ie && /Apple Computer/.test(nav.vendor)
const ios =
  safari && (/Mobile\/\w+/.test(nav.userAgent) || nav.maxTouchPoints > 2)
const isMetaKey = ios || /Mac/.test(nav.platform)

interface ActionMarkedEvent {
  metaKey: boolean
  ctrlKey: boolean
}

interface AltMarkedEvent {
  altKey: boolean
  ctrlKey: boolean
}

export const actionKey = isMetaKey ? 'Meta' : 'Control'

export function hasActionKey(event: ActionMarkedEvent): boolean {
  return isMetaKey ? event.metaKey : event.ctrlKey
}

export function hasAltKey(event: AltMarkedEvent): boolean {
  return event.altKey || (!isMetaKey && event.ctrlKey)
}
