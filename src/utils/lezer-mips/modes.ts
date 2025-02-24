import { Compartment } from '@codemirror/state'
import { showMinimap } from '@replit/codemirror-minimap'
import { vim as vimSetup } from '@replit/codemirror-vim'
import { EditorView } from 'codemirror'

const vimCompartment = new Compartment()
const minimapCompartment = new Compartment()
const editorTheme = new Compartment()

const vim = vimSetup()

export const setVim = (value: boolean) =>
  vimCompartment.reconfigure(value ? vim : [])

export const createDefaultVim = () => vimCompartment.of([])

const minimap = showMinimap.compute(['doc'], () => {
  return {
    create: () => {
      const dom = document.createElement('div')
      return { dom }
    },
    displayText: 'blocks',
    showOverlay: 'always',
  }
})

export const setMinimap = (value: boolean) =>
  minimapCompartment.reconfigure(value ? minimap : [])

export const createDefaultMinimap = () => minimapCompartment.of(minimap)

// defined in codemirror.css
const darkTheme = EditorView.theme({}, { dark: true })
const lightTheme = EditorView.theme({}, { dark: false })

export const setTheme = (theme: boolean) =>
  editorTheme.reconfigure(theme ? darkTheme : lightTheme)

export const createDefaultTheme = () => editorTheme.of(lightTheme)
