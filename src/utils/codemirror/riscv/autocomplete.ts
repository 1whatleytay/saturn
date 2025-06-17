import { CompletionContext } from '@codemirror/autocomplete'
import { suggestions } from '../suggestions'
import { SuggestionType } from '../../languages/suggestions'

export function myCompletions(context: CompletionContext) {
  // if we're in a string token, don't show completions
  if (context.tokenBefore(['String'])) {
    return null
  }

  // because line comments sometimes don't show up as tokens in the tree,
  // tokenBefore only detects Mips as the token instead of LineComment
  if (context.matchBefore(/#.*/)) {
    return null
  }

  let word = context.matchBefore(/[a-zA-Z$._]*/)!
  if (word.from == word.to && !context.explicit) return null

  let labels: { detail: string; label: string; type: string | undefined }[] = []

  const suggestionsContext = context.view?.state.field(suggestions)

  if (suggestionsContext) {
    const iter = suggestionsContext.iter()

    while (iter.value) {
      const suggestion = iter.value.suggestion

      labels.push({
        detail: suggestion.name ?? suggestion.replace,
        label: suggestion.replace,
        type: (() => {
          switch (suggestion.type) {
            case SuggestionType.Label:
              return 'label' // ?
            case SuggestionType.Function:
              return 'function'
            case SuggestionType.Variable:
              return 'constant'
            default:
              return undefined
          }
        })(),
      })

      iter.next()
    }
  }

  return {
    from: word.from,
    options: [
      ...[
        { detail: 'Zero Register', label: 'x0' },
      ].map((x) => ({ ...x, type: 'register' })),
      ...[
        { detail: 'Ascii Text', label: '.ascii' },
        { detail: 'Ascii Zero Terminated', label: '.asciiz' },
        { detail: 'Align Bytes', label: '.align' },
        { detail: 'Space Bytes', label: '.space' },
        { detail: 'Byte Literals', label: '.byte' },
        { detail: 'Half Literals', label: '.half' },
        { detail: 'Word Literals', label: '.word' },
        { detail: 'Float Literals', label: '.float' },
        { detail: 'Double Literals', label: '.double' },
        { detail: 'Entry Point', label: '.entry' },
        { detail: 'Text Section', label: '.text' },
        { detail: 'Data Section', label: '.data' },
        { detail: 'Kernel Text Section', label: '.ktext' },
        { detail: 'Kernel Data Section', label: '.kdata' },
        { detail: 'Extern Symbol', label: '.extern' },
        { detail: 'Define Token', label: '.eqv' },
        { detail: 'Define Macro', label: '.macro' },
        { detail: 'End Macro', label: '.end_macro' },
        { detail: 'Include Source', label: '.include' },
      ].map((x) => ({ ...x, type: 'data' as string | undefined })),
      ...[
        { detail: 'Shift Left', label: 'sll' },
      ].map((x) => ({ ...x, type: 'instruction' })),
      ...labels,
    ],
  }
}
