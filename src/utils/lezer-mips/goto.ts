import { EditorView } from 'codemirror'
import { RangeSet, StateEffect, StateField } from '@codemirror/state'
import { Decoration, DecorationSet, WidgetType } from '@codemirror/view'
import { actionKey, hasActionKey } from '../query/shortcut-key'
import { suggestions } from './suggestions'
import { MipsHighlighter } from '../languages/mips/language'
import { SuggestionType } from '../languages/suggestions'
import { grabWhitespace } from '../languages/language'
import { suggestionLetter, suggestionStyle } from '../query/suggestion-styles'

interface GotoState {
  inspecting?: boolean // true if we are holding down the action key
  pos?: number
}

interface GotoDestination {
  destLine: number
  destPos: number
  srcPos: number
  name: string
  type?: SuggestionType
}

class GotoWidgetType extends WidgetType {
  constructor(public destination: GotoDestination) {
    super()
  }

  toDOM(_view: EditorView): HTMLElement {
    // Using Vue <Teleport /> here seems challenging.
    // Specifically, passing this.destination over to the Vue stack.
    // There is probably a way to pass through to Teleport, or at least something nicer than this.
    // Worth changing in the future. I generally didn't want to use global state to pass around destination.

    const result = document.createElement('span')

    result.className = 'relative'

    const inner = document.createElement('div')

    inner.className =
      'absolute pointer-events-none left-0 py-2 px-4 w-auto dark:bg-neutral-700 bg-neutral-300 rounded shadow-xl z-30 dark:text-gray-300 text-gray-800 font-medium font-sans flex items-center'

    const label = document.createElement('span')
    label.className = 'dark:text-gray-200 text-gray-800 font-bold font-mono'
    label.innerText = this.destination.name ?? 'label'

    const marker = document.createElement('div')

    const markerClasses = suggestionStyle(this.destination.type)

    marker.className = `rounded ml-4 w-4 h-4 text-black font-black flex items-center justify-center text-xs shrink-0 ${markerClasses}`
    marker.innerText = suggestionLetter(this.destination.type)

    inner.appendChild(document.createTextNode('Jump to '))
    inner.appendChild(label)
    inner.appendChild(
      document.createTextNode(` (line ${this.destination.destLine})`),
    )
    inner.appendChild(marker)

    result.appendChild(inner)

    return result
  }
}

function gotoWidget(destination: GotoDestination) {
  return Decoration.widget({
    widget: new GotoWidgetType(destination),
    side: 1,
  })
}

const marker = Decoration.mark({
  class: 'underline decoration-dotted',
})

const gotoEffect = StateEffect.define<GotoDestination | null>()

const gotoDecoration = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(gotoEffect)) {
        if (effect.value) {
          value = RangeSet.of([
            gotoWidget(effect.value).range(effect.value.srcPos),
            marker.range(
              effect.value.srcPos,
              effect.value.srcPos + effect.value.name.length,
            ),
          ])
        } else {
          value = Decoration.none
        }
      }
    }

    return value
  },
  provide: (f) => EditorView.decorations.from(f),
})

// Horrible way of approaching this.
// Basically, I'd like to know which token I am hovering over.
// I could write a function to move to the left/right of the token.
// But I'd like to try to stay consistent with how our tokenizer works today.
// ... and I don't want to rewrite that code right now.
// This should be replaced in the future with something more isolated/better.
const highlighter = new MipsHighlighter()

function checkForGotoDestination(
  pos: number,
  view: EditorView,
): GotoDestination | null {
  const insights = view.state.field(suggestions)

  if (!insights) {
    return null
  }

  const line = view.state.doc.lineAt(pos)
  const linePos = pos - line.from
  const { tokens } = highlighter.highlight(line.text)

  const token = tokens.find(
    (token) =>
      linePos >= token.start && linePos < token.start + token.text.length,
  )

  if (!token) {
    return null
  }

  // Accounting for really poor tokenization.
  const { leading } = grabWhitespace(token.text)

  const text = token.text.trim()

  const iter = insights.iter()

  while (iter.value) {
    if (iter.value.suggestion.replace === text) {
      const destLine = view.state.doc.lineAt(iter.from)

      return {
        destPos: iter.from,
        destLine: destLine.number,
        srcPos: token.start + line.from + leading.length,
        name: iter.value.suggestion.replace,
        type: iter.value.suggestion.type,
      }
    }

    iter.next()
  }

  return null
}

export const goto = [
  gotoDecoration,
  // `this` seems to persist between events, but does not seem to be the `handlers` object passed to this call.
  // As long as I am careful I can use it to track the mouse position.
  EditorView.domEventHandlers({
    keydown(this: GotoState, event, view) {
      if (event.key === actionKey && this.pos !== undefined) {
        this.inspecting = true

        view.dispatch({
          effects: gotoEffect.of(checkForGotoDestination(this.pos, view)),
        })
      }
    },

    keyup(this: GotoState, event, view) {
      if (event.key === actionKey) {
        this.inspecting = false

        view.dispatch({
          effects: gotoEffect.of(null),
        })
      }
    },

    mousedown(this: GotoState, event, view) {
      if (hasActionKey(event) && this.pos !== undefined) {
        const result = checkForGotoDestination(this.pos, view)

        if (result) {
          view.dispatch({
            selection: {
              anchor: result.destPos,
            },
            scrollIntoView: true,
          })

          return true
        }
      }
    },

    mousemove(this: GotoState, event, view) {
      this.pos =
        view.posAtCoords({ x: event.clientX, y: event.clientY }) ?? undefined

      if (this.pos !== undefined) {
        if (hasActionKey(event)) {
          this.inspecting = true
          view.dispatch({
            effects: gotoEffect.of(checkForGotoDestination(this.pos, view)),
          })
        } else if (this.inspecting) {
          this.inspecting = false
          view.dispatch({
            effects: gotoEffect.of(null),
          })
        }
      }
    },

    mouseleave(this: GotoState, _event, _view) {
      this.pos = undefined
    },

    // Ignoring focusout events.
  }),
]
