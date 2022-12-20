interface LineData {
  lines: string[]
}

interface Cursor {
  line: number
  index: number
}

interface UndoProvider {
  undo(data: LineData): void
}

class FullFrame implements UndoProvider {
  undo(data: LineData): void {
    data.lines = this.lines // .slice() ?
  }

  constructor(
    public lines: string[]
  ) { }
}

type Operation = FullFrame
type Step = { op: Operation, cursor: Cursor }

export class UndoHistory {
  current: Step | null = null
  operations: Step[] = []

  timeout: number | null = null
  uncommitted: number = 0

  lastLines: LineData | null = null

  private commit() {
    this.uncommitted = 0

    if (this.current) {
      console.log('committing')

      this.operations.push(this.current)

      if (this.operations.length > this.backlog) {
        this.operations = this.operations.splice(0, this.operations.length - this.backlog)
      }

      this.current = null
    }
  }

  private lines(): LineData {
    const result = this.data()

    // Heavy mutation here.
    if (result !== this.lastLines) {
      this.lastLines = result
      this.clear() // might need to be reworked
    }

    return result
  }

  public push(step: Step) {
    // Assuming merging was properly done.
    this.current = step
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

  public frame() {
    const value = {
      lines: this.lines().lines.slice()
    }

    if (this.current) {
      this.current.op.undo(value)
    }

    const cursorCopy = () => {
      const cursor = this.getCursor()

      return {
        line: cursor.line,
        index: cursor.index
      }
    }

    this.push({
      op: new FullFrame(value.lines),
      cursor: this.current?.cursor ?? cursorCopy()
    })
  }

  public undo() {
    let operation: Step

    if (this.current) {
      operation = this.current

      this.current = null
    } else {
      const op = this.operations.pop()

      if (!op) {
        return
      }

      operation = op
    }

    operation.op.undo(this.lines())
    this.setCursor(operation.cursor)
  }

  public clear() {
    this.current = null
    this.operations = []
  }

  constructor(
    private data: () => LineData,
    private getCursor: () => Cursor,
    private setCursor: (cursor: Cursor) => void,
    private backlog: number = 50,
    private debounce: number = 800,
    private commitInterval: number = 30
  ) { }
}
