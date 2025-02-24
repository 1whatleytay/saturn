import {
  StateEffect,
  StateField,
  Text,
  Range,
  RangeValue,
  RangeSet,
} from '@codemirror/state'
import { EditorView } from 'codemirror'
import { MipsHighlighter } from '../languages/mips/language'
import { Suggestion } from '../languages/suggestions'

class InsightValue extends RangeValue {
  eq(other: RangeValue): boolean {
    return (
      other instanceof InsightValue &&
      this.suggestion.replace === other.suggestion.replace &&
      this.suggestion.name === other.suggestion.name &&
      this.suggestion.type === other.suggestion.type
    )
  }

  constructor(public suggestion: Suggestion) {
    super()

    this.point = false
  }
}

interface InsightElement {
  value: InsightValue
  index: number
  count: number
}

interface InsightUpdate {
  startLineIndex: number
  endLineIndex: number

  elements: InsightElement[]
}

const insightUpdateEffect = StateEffect.define<InsightUpdate>()

// Used for deriving suggestions. Should probably be replaced with our syntax.grammar lexer soon.
// Don't really see a reason the syntax.grammar highlighter can't be used here.
const highlighter = new MipsHighlighter()

function inspect(
  startLine: number,
  endLine: number,
  doc: Text,
): InsightElement[] {
  const postCount = endLine - startLine + 1

  const elements: InsightElement[] = []

  for (let a = 0; a < postCount; a++) {
    // Hopefully no OOB here.
    const line = startLine + a

    const details = doc.line(line)

    const result = highlighter.highlight(details.text)

    elements.push(
      ...result.suggestions.map((suggestion) => ({
        index: suggestion.index + details.from,
        count: suggestion.replace.length,
        value: new InsightValue(suggestion),
      })),
    )
  }

  return elements
}

export const suggestions = StateField.define<RangeSet<InsightValue> | null>({
  create() {
    // Assumption: Switching tabs will also create new StateFields (it almost definitely does).
    return null
  },
  update(value, tr) {
    function elementToRange({
      index,
      count,
      value,
    }: InsightElement): Range<InsightValue> {
      return value.range(index, index + count)
    }

    if (value === null) {
      const elements = inspect(
        1,
        tr.startState.doc.lines,
        tr.startState.doc,
      ).map(elementToRange)

      // Is the end position inclusive?
      value = RangeSet.of(elements, true)
    }

    // We are applying the effects before applying the transaction changes.
    // This is for an important reason that I'm not sure.
    for (const effect of tr.effects) {
      if (effect.is(insightUpdateEffect)) {
        console.assert(!tr.docChanged)

        const insight: InsightUpdate = effect.value

        value = value.update({
          filter(from, to) {
            // console.log({ from, to, sli: insight.startLineIndex, eli: insight.endLineIndex, keep: !(from <= insight.endLineIndex && to >= insight.startLineIndex) })
            // If it intersects with this region in any way.
            return !(
              from <= insight.endLineIndex && to >= insight.startLineIndex
            )
          },
          add: insight.elements.map(elementToRange),
          sort: true,
        })
      }
    }

    value = value.map(tr.changes)

    return value
  },
})

export const suggestionsContext = [
  EditorView.updateListener.of((update) => {
    if (!update.docChanged) {
      return
    }

    const insights: InsightUpdate[] = []

    update.changes.iterChanges((_fromA, _toA, fromB, toB) => {
      const startLine = update.state.doc.lineAt(fromB)
      const endLine = update.state.doc.lineAt(toB)

      // we can pass these details here to an LSP if needed, we probably just want to debounce the request
      // here we will deal with them synchronously

      const elements = inspect(
        startLine.number,
        endLine.number,
        update.state.doc,
      )

      insights.push({
        startLineIndex: startLine.from,
        endLineIndex: endLine.to,
        elements,
      })
    })

    // Assumption here: Effects are delivered timely.
    update.view.dispatch({
      effects: insights.map((insight) => insightUpdateEffect.of(insight)),
    })
  }),
  suggestions,
]
