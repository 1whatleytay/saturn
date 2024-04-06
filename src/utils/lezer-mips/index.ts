import { parser } from './syntax.grammar'
import {
  LRLanguage,
  LanguageSupport,
  foldNodeProp,
  foldInside,
} from '@codemirror/language'
import { styleTags, tags as t } from '@lezer/highlight'
import { CompletionContext, autocompletion } from '@codemirror/autocomplete'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { StateEffect, StateField } from '@codemirror/state'
import { Decoration, DecorationSet, EditorView } from '@codemirror/view'

export const MipsLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      foldNodeProp.add({
        Application: foldInside,
      }),
      styleTags({
        Identifier: t.variableName,
        Label: t.attributeName,
        //args: t.bool,
        Register: t.typeName,
        Number: t.number,
        String: t.string,
        Op: t.keyword,
        Macro: t.macroName,
        LineComment: t.lineComment,
      }),
    ],
  }),
  languageData: {
    commentTokens: { line: ';' },
    autocomplete: function myCompletions(context: CompletionContext) {
      let word = context.matchBefore(/\w*/)!
      if (word.from == word.to && !context.explicit) return null
      return {
        from: word.from,
        options: [
          { label: 'match', type: 'keyword' },
          { label: 'hello', type: 'variable', info: '(World)' },
          { label: 'magic', type: 'text', apply: '⠁⭒*.✩.*⭒⠁', detail: 'macro' },
        ],
      }
    },
  },
})

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
    new LanguageSupport(MipsLanguage),
    autocompletion({ activateOnTyping: true }),
    highlightedLineState,
  ]
}
