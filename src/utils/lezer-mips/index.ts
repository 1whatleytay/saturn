import { tags as t } from '@lezer/highlight'
import { autocompletion } from '@codemirror/autocomplete'
import {
  syntaxHighlighting,
  HighlightStyle,
  indentService,
} from '@codemirror/language'
import { StateEffect, StateField } from '@codemirror/state'
import { Decoration, DecorationSet, EditorView } from '@codemirror/view'
import { lang } from './lezer-lang'
import { foldOnIndent } from './folding'
import './codemirror.css'
import { goto } from './goto'

export const clearHighlightedLine = StateEffect.define<null>()
export const setHighlightedLine = StateEffect.define<number>()

const pausedLine = Decoration.line({
  attributes: {
    class: 'dark:bg-breakpoint-stopped bg-breakpoint-stopped-light',
  },
})

const highlightedLineState = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    if (!tr.changes.empty && value.size) {
      value = value.map(tr.changes)
    }
    for (const effect of tr.effects) {
      if (effect.is(clearHighlightedLine)) {
        value = Decoration.none
      } else if (effect.is(setHighlightedLine)) {
        value = Decoration.set([pausedLine.range(effect.value)])
      }
    }
    return value
  },
  provide: (field) => EditorView.decorations.from(field),
})

const twHighlightStyle = HighlightStyle.define([
  { tag: [t.variableName], class: 'dark:text-white text-black' },
  { tag: [t.attributeName], class: 'dark:text-amber-400 text-amber-700' },
  { tag: [t.typeName], class: 'dark:text-orange-300 text-orange-800' },
  { tag: [t.number], class: 'dark:text-teal-300 text-teal-800' },
  { tag: [t.string], class: 'dark:text-lime-300 text-lime-800' },
  { tag: [t.keyword], class: 'dark:text-sky-400 text-sky-700' },
  { tag: [t.macroName], class: 'dark:text-red-400 text-red-700' },
  { tag: [t.lineComment], class: 'dark:text-neutral-400 text-neutral-500' },
])

export function Mips() {
  return [
    syntaxHighlighting(twHighlightStyle),
    lang,
    autocompletion({ activateOnTyping: true }),
    highlightedLineState,
    indentService.of((context, pos) => {
      const previousLine = context.lineAt(pos, -1)
      const prevIndent = context.lineIndent(pos, -1)
      if (previousLine.text.match(/:$/)) return prevIndent + context.unit
      return prevIndent
    }),
    foldOnIndent,
    goto,
  ]
}
