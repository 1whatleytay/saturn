import {StateField, StateEffect, RangeSet, EditorState} from "@codemirror/state"
import {Decoration, EditorView, gutter, gutterLineClass, GutterMarker, lineNumbers} from "@codemirror/view"
import { setBreakpoint } from "./debug";

const breakpointEffect = StateEffect.define<{pos: number, on: boolean}>({
  map: (val, mapping) => ({pos: mapping.mapPos(val.pos), on: val.on})
})

const breakpointedLine = Decoration.line({
  attributes: {
    class: 'dark:bg-breakpoint-neutral bg-breakpoint-neutral-light',
  },
});

const breakpointState = StateField.define<RangeSet<GutterMarker>>({
  create: () => RangeSet.empty,
  update(set, transaction) {
    set = set.map(transaction.changes)
    for (let e of transaction.effects) {
      if (e.is(breakpointEffect)) {
        if (e.value.on)
          set = set.update({add: [breakpointMarker.range(e.value.pos)]})
        else
          set = set.update({filter: from => from != e.value.pos})
      }
    }
    return set
  }
})

const breakpointLines = StateField.define<RangeSet<Decoration>>({
  create: () => Decoration.none,
  update(set, transaction) {
    set = set.map(transaction.changes)
    for (let e of transaction.effects) {
      if (e.is(breakpointEffect)) {
        if (e.value.on)
          set = set.update({add: [breakpointedLine.range(e.value.pos)] })
        else
          set = set.update({filter: from => from != e.value.pos})
      }
    }
    return set
  },
  provide: f => EditorView.decorations.from(f)
})


function toggleBreakpoint(view: EditorView, pos: number) {
  let breakpoints = view.state.field(breakpointState)
  let hasBreakpoint = false
  breakpoints.between(pos, pos, () => {hasBreakpoint = true})
  view.dispatch({
    effects: breakpointEffect.of({pos, on: !hasBreakpoint})
  })
}

export function getBreakpoints(state: EditorState) {
  let breakpoints = state.field(breakpointState)
  let result: number[] = []
  breakpoints.between(0, state.doc.length, from => {result.push(state.doc.lineAt(from).number-1)})
  return result
}
const breakpointMarker = new class extends GutterMarker {
  elementClass = "cm-breakpoint"
}

export const breakpointGutter = [
  breakpointState,
  breakpointLines,
  EditorView.baseTheme({
    ".cm-lineNumbers .cm-gutterElement": {
      "padding-left": "0.75em",
      "position": "relative",
    },
    ".cm-lineNumbers .cm-gutterElement:hover::before": {
      opacity: 0.5,
    },
    ".cm-lineNumbers .cm-breakpoint::before, .cm-lineNumbers .cm-breakpoint:hover::before": {
      opacity: 1,
    },
    ".cm-lineNumbers .cm-breakpoint::before, .cm-lineNumbers .cm-gutterElement:hover::before": {
      content: '"●"',
      color: "red",
      cursor: "pointer",
      position: "absolute",
      left: "0.125em"
    },
  }),
  lineNumbers({
    domEventHandlers: {
      mousedown(view, line) {
        toggleBreakpoint(view, line.from)
        return true
      }
    }
  }),
  gutterLineClass.from(breakpointState),
]
