import { tags as t } from '@lezer/highlight'
import { autocompletion } from '@codemirror/autocomplete'
import { syntaxHighlighting, HighlightStyle, foldGutter } from '@codemirror/language'
import { StateEffect, StateField } from '@codemirror/state'
import { Decoration, DecorationSet, EditorView } from '@codemirror/view'
import { lang } from './lezer-lang'

export const clearHighlightedLine = StateEffect.define<null>()
export const setHighlightedLine = StateEffect.define<number>()

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
        value = Decoration.set([
          Decoration.line({ attributes: { class: 'bg-red-800' } }).range(
            effect.value,
          ),
        ])
      }
    }
    return value
  },
  provide: (field) => EditorView.decorations.from(field, (value) => value),
})

const cursor = '#fb923c'

const twHighlightStyle = HighlightStyle.define([
  { tag: [t.variableName], class: 'text-white' },
  { tag: [t.attributeName], class: 'text-amber-400' },
  { tag: [t.typeName], class: 'text-orange-300' },
  { tag: [t.number], class: 'text-teal-300' },
  { tag: [t.string], class: 'text-lime-300' },
  { tag: [t.keyword], class: 'text-sky-400' },
  { tag: [t.macroName], class: 'text-red-400' },
  { tag: [t.lineComment], class: 'text-neutral-400' },
])

const darkTheme = EditorView.theme(
  {
    '.cm-content': {
      caretColor: cursor,
    },

    '.cm-cursor, .cm-dropCursor': {
      borderLeftColor: cursor,
      borderLeftWidth: '2px',
    },
  },
  { dark: true },
)

export function Mips() {
  return [
    syntaxHighlighting(twHighlightStyle),
    darkTheme,
    foldGutter(),
    lang,
    autocompletion({ activateOnTyping: true }),
    highlightedLineState,
  ]
}
