import { parser } from './syntax.grammar'
import {
  LRLanguage,
  LanguageSupport,
  indentNodeProp,
  foldNodeProp,
  foldInside,
  delimitedIndent,
} from '@codemirror/language'
import { styleTags, tags as t } from '@lezer/highlight'

export const MipsLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      foldNodeProp.add({
        Application: foldInside,
      }),
      styleTags({
        Identifier: t.variableName,
        "Label": t.bool,
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
  },
})

export function Mips() {
  return new LanguageSupport(MipsLanguage)
}
