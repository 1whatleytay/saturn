import { Compartment } from '@codemirror/state'
import { showMinimap } from '@replit/codemirror-minimap'
import { vim as vimSetup, Vim } from '@replit/codemirror-vim'
import { EditorView } from 'codemirror'

export const vimCompartment = new Compartment()
export const minimapCompartment = new Compartment()

export const vim = vimSetup()

export const setVim = (value: boolean) =>
  vimCompartment.reconfigure(value ? vim : [])

export const minimap = showMinimap.compute(['doc'], (state) => {
  return {
    create: (v: EditorView) => {
      const dom = document.createElement('div')
      return { dom }
    },
  }
})

export const setMinimap = (value: boolean) =>
  minimapCompartment.reconfigure(value ? minimap : [])
