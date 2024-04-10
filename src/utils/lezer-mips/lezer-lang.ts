import { parser } from './syntax.grammar'
import {
  LRLanguage,
  LanguageSupport,
  foldNodeProp,
  indentNodeProp,
} from '@codemirror/language'
import { styleTags, tags as t } from '@lezer/highlight'
import { CompletionContext } from '@codemirror/autocomplete'
import { myCompletions } from './autocomplete'

const MipsLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      foldNodeProp.add({
        LabelGroup: (node) => {
          let first = node.firstChild
          return first && first.to ? {from: first.to, to: node.to} : null
        },
      }),
      indentNodeProp.add({
        LabelGroup: context => context.column(context.node.from) + context.unit
      }),
      styleTags({
        Identifier: t.variableName,
        Label: t.attributeName,
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
    commentTokens: { line: '#' },
    autocomplete: myCompletions,
  },
})

export const lang = new LanguageSupport(MipsLanguage)
