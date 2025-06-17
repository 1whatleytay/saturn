import { indentUnit } from '@codemirror/language'
import { Compartment, EditorState } from '@codemirror/state'
import { showMinimap } from '@replit/codemirror-minimap'
import { vim as vimSetup } from '@replit/codemirror-vim'
import { EditorView } from 'codemirror'
import { lexer } from './suggestions'
import { lex as mipslex } from '../languages/mips/lexer'
import { lex as riscvlex } from '../languages/risc-v/lexer'
import { lang as mipslang } from './mips'
import { lang as riscvlang } from './riscv'

const vimCompartment = new Compartment()
const minimapCompartment = new Compartment()
const editorTheme = new Compartment()
const indentUnitCompartment = new Compartment()
const langCompartment = new Compartment()

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

export const createIndentUnit = () => indentUnitCompartment.of([])

const mips = [lexer.of(mipslex), mipslang];
const riscv = [lexer.of(riscvlex), riscvlang];
export const createDefaultLang = () => langCompartment.of(mips)
export const setLang = (lang: 'mips' | 'riscv') =>
  langCompartment.reconfigure(lang === 'mips' ? mips : riscv)

export const setIndentUnit = (unit: number) =>
  indentUnitCompartment.reconfigure([
    indentUnit.of(' '.repeat(unit)),
    EditorState.tabSize.of(unit),
  ])
