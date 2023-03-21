import { consumeBackwards, consumeForwards, consumeSpace } from './query/alt-consume'
import { grabWhitespace } from './languages/language'
import { splitLines } from './split-lines'

export interface SelectionIndex {
  line: number,
  index: number
}

export interface SelectionRange {
  startLine: number
  startIndex: number
  endLine: number
  endIndex: number
}

export interface LineRange {
  start: number
  end: number
}

type LineData = string[]

interface Frame {
  index: number,
  deleted: string[],
  replaced: number
}

type Step = { frame: Frame, cursor: SelectionIndex }

interface MergeResult {
  commit?: Frame
  merged: Frame
}

function collides(s1: number, e1: number, s2: number, e2: number) {
  return s1 <= e2 && s2 <= e1
}

function merge(last: Frame, next: Frame): MergeResult {
  const lastStart = last.index
  const lastEnd = last.index + last.replaced
  const nextStart = next.index
  const nextEnd = next.index + next.deleted.length

  // Common Case
  if (lastStart == nextStart
    && last.deleted.length == next.deleted.length
    && last.replaced == next.replaced) {
    return { merged: last }
  }

  // Regular Case
  if (!collides(lastStart, lastEnd, nextStart, nextEnd)) {
    return { commit: last, merged: next }
  }

  const leadingSize = Math.max(0, last.index - next.index)
  const collidingSize = Math.max(0, Math.min(lastEnd, nextEnd) - Math.max(lastStart, nextStart))
  const leading = next.deleted.slice(0, leadingSize)
  const trailing = next.deleted.slice(leadingSize + collidingSize)

  const replaced = last.replaced + next.replaced - collidingSize
  const deleted = leading.concat(last.deleted).concat(trailing)

  const start = Math.min(last.index, next.index)

  return { merged: { index: start, deleted, replaced } }
}

export type DirtyHandler = (line: number, deleted: number, insert: string[]) => void

export class Editor {
  current: Step | null = null
  operations: Step[] = []

  timeout: number | null = null
  uncommitted: number = 0

  lastLines: LineData | null = null

  public commit() {
    this.uncommitted = 0

    if (this.current) {
      this.operations.push(this.current)

      if (this.operations.length > this.backlog) {
        this.operations = this.operations.splice(
          0, this.operations.length - this.backlog
        )
      }

      this.current = null
    }
  }

  public push(step: Step) {
    if (this.current) {
      let cursor = this.current.cursor
      const { commit, merged } = merge(this.current.frame, step.frame)

      if (commit) {
        this.current = { cursor, frame: commit }
        this.commit()

        cursor = step.cursor
      }

      this.current = { cursor, frame: merged }
    } else {
      this.current = step
    }

    this.uncommitted++

    if (this.timeout) {
      window.clearTimeout(this.timeout)
    }

    if (this.uncommitted > this.commitInterval) {
      this.commit()
    } else {
      this.timeout = window.setTimeout(() => {
        this.commit()
      }, this.debounce)
    }
  }

  public mergedCursor(): SelectionIndex {
    // Not moved, kinda freaky!
    const lastCursor = this.current?.cursor

    if (lastCursor) {
      return lastCursor
    } else {
      return {
        line: this.cursor.line,
        index: this.cursor.index
      }
    }
  }

  public dirty(line: number, count: number, insert?: number) {
    const backup = this.data.slice(line, line + count)

    // replace with self
    this.push({
      frame: { index: line, deleted: backup, replaced: insert ?? count },
      cursor: this.mergedCursor()
    })
  }

  public mutate(line: number, count: number, insert: number, body: () => void) {
    if (this.writable && !this.writable(line, count, insert)) {
      return
    }

    this.dirty(line, count, insert)

    body()

    this.onDirty(line, count, this.data.slice(line, line + insert))
  }

  public mutateLine(line: number, body: () => void) {
    this.mutate(line, 1, 1, body)
  }

  popStep(): Step | null {
    if (this.current) {
      const result = this.current

      this.current = null

      return result
    } else {
      return this.operations.pop() ?? null
    }
  }

  public undo(): SelectionIndex | null {
    const step = this.popStep()

    if (!step) {
      return null
    }

    const frame = step.frame

    this.data.splice(frame.index, frame.replaced, ...frame.deleted)
    this.onDirty(frame.index, frame.replaced, frame.deleted)

    return step.cursor
  }

  public clear() {
    this.current = null
    this.operations = []
  }

  drop(range: SelectionRange) {
    if (range.startLine == range.endLine) {
      const text = this.data[range.startLine]

      const leading = text.substring(0, range.startIndex)
      const trailing = text.substring(range.endIndex)

      this.mutateLine(range.startLine, () => {
        this.data[range.startLine] = leading + trailing
      })
    } else {
      const leading = this.data[range.startLine].substring(0, range.startIndex)
      const trailing = this.data[range.endLine].substring(range.endIndex)

      this.mutate(range.startLine, range.endLine - range.startLine + 1, 1, () => {
        this.data[range.startLine] = leading + trailing
        this.data.splice(range.startLine + 1, range.endLine - range.startLine)
      })
    }
  }

  prefix(start: number, end: number, character: string, whitespace: boolean = false) {
    const count = end - start + 1

    this.mutate(start, count, count, () => {
      for (let a = start; a <= end; a++) {
        if (whitespace) {
          const text = this.data[a]
          const { leading } = grabWhitespace(text)

          // Ignore empty strings when whitespace matters
          if (leading.length !== text.length) {
            this.data[a] = leading + character + text.substring(leading.length)
          }
        } else {
          this.data[a] = character + this.data[a]
        }
      }
    })
  }

  crop(start: number, ranges: (LineRange | null)[]) {
    this.mutate(start, ranges.length, ranges.length, () => {
      ranges.forEach((range, index) => {
        if (!range) {
          return
        }

        const line = start + index
        const text = this.data[line]

        this.data[line] = text.substring(0, range.start) + text.substring(range.end)
      })
    })
  }

  put(index: SelectionIndex, character: string): SelectionIndex {
    const line = this.data[index.line]
    const leading = line.substring(0, index.index)
    const trailing = line.substring(index.index)

    // Mutate
    this.mutateLine(index.line, () => {
      this.data[index.line] = leading + character + trailing
    })

    return { line: index.line, index: index.index + character.length }
  }

// Technical Debt: insert vs paste (concern: speed for splitting by \n)
  paste(index: SelectionIndex, text: string): SelectionIndex {
    const textLines = splitLines(text)

    if (!textLines.length) {
      return index
    }

    if (textLines.length === 1) {
      return this.put(index, text)
    }

    // at least two lines
    const line = this.data[index.line]
    const leading = line.substring(0, index.index)
    const trailing = line.substring(index.index)

    const first = textLines[0]
    const rest = textLines.slice(1)
    const last = rest[rest.length - 1].length
    rest[rest.length - 1] += trailing

    // Mutate
    this.mutate(index.line, 1, textLines.length, () => {
      this.data[index.line] = leading + first
      this.data.splice(index.line + 1, 0, ...rest)
    })

    return { line: index.line + textLines.length - 1, index: last }
  }

  dropTab(line: number, spacing: number): number {
    const regex = new RegExp(`^( {1,${spacing}}|\\t)`, 'g')
    const match = this.data[line].match(regex)

    if (match && match.length) {
      const text = match[0]

      this.mutateLine(line, () => {
        this.data[line] = this.data[line].substring(text.length)
      })

      return text.length
    }

    return 0
  }

  newline(index: SelectionIndex): SelectionIndex {
    const line = this.data[index.line]
    const leading = line.substring(0, index.index)
    const trailing = line.substring(index.index)

    const leadingSpacing = /^\s*/g
    const match = line.match(leadingSpacing)
    const endMatch = trailing.match(leadingSpacing)

    const spacing = match && match.length ? match[0] : ''
    const noSpace = endMatch && endMatch.length ? trailing.substring(endMatch[0].length) : trailing

    // Mutate
    this.mutate(index.line, 1, 2, () => {
      this.data[index.line] = leading
      this.data.splice(index.line + 1, 0, spacing + noSpace)
    })

    return { line: index.line + 1, index: spacing.length }
  }

  backspace(index: SelectionIndex, alt: boolean = false, space: number = 1): SelectionIndex {
    const line = this.data[index.line]

    if (index.index > 0) {
      const consumption = alt
        ? consumeBackwards(line, index.index)
        : consumeSpace(line, index.index, -1, space)

      const leading = line.substring(0, index.index - consumption)
      const trailing = line.substring(index.index)

      // Mutate
      this.mutateLine(index.line, () => {
        this.data[index.line] = leading + trailing
      })

      return { line: index.line, index: leading.length }
    } else if (index.line > 0) {
      const leading = this.data[index.line - 1]
      const trailing = this.data[index.line]

      this.mutate(index.line - 1, 2, 1, () => {
        this.data[index.line - 1] = leading + trailing
        this.data.splice(index.line, 1)
      })

      return { line: index.line - 1, index: leading.length }
    }

    return index
  }

  deleteForwards(index: SelectionIndex, alt: boolean = false): SelectionIndex {
    const line = this.data[index.line]

    if (index.index < line.length) {
      const consumption = alt ? consumeForwards(line, index.index) : 1

      const leading = line.substring(0, index.index)
      const trailing = line.substring(index.index + consumption)

      // Mutate
      this.mutateLine(index.line, () => {
        this.data[index.line] = leading + trailing
      })

      return { line: index.line, index: leading.length }
    } else if (index.line + 1 < this.data.length) {
      const leading = this.data[index.line]
      const trailing = this.data[index.line + 1]

      this.mutate(index.line, 2, 1, () => {
        this.data[index.line] = leading + trailing
        this.data.splice(index.line + 1, 1)
      })

      return { line: index.line, index: leading.length }
    }

    return index
  }

  // Immutable Line Methods
  grab(range: SelectionRange): string {
    // assert range.startLine >= range.endLine
    if (range.startLine == range.endLine) {
      const text = this.data[range.startLine]

      return text.substring(range.startIndex, range.endIndex)
    } else {
      const leading = this.data[range.startLine].substring(range.startIndex)
      const middle = this.data.slice(range.startLine + 1, range.endLine)
      const trailing = this.data[range.endLine].substring(0, range.endIndex)

      return [leading, ...middle, trailing].join('\n')
    }
  }

  lineCount(): number {
    return this.data.length
  }

  lineAt(index: number): string {
    return this.data[index]
  }

  constructor(
    public data: LineData,
    public cursor: SelectionIndex,
    public onDirty: DirtyHandler = () => { },
    public writable?: (start: number, deleted: number, insert: number) => boolean, // end - not inclusive
    public backlog: number = 50,
    public debounce: number = 800,
    public commitInterval: number = 30
  ) { }
}
