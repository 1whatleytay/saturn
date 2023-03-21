import { getStyle, HighlightResult, Token, TokenType } from '../language'
import { MarkedSuggestion, SuggestionType } from '../suggestions'

enum ItemSuggestionMarker {
  Macro,
  Eqv
}

function toSuggestionType(marker: ItemSuggestionMarker): SuggestionType {
  switch (marker) {
    case ItemSuggestionMarker.Eqv: return SuggestionType.Variable
    case ItemSuggestionMarker.Macro: return SuggestionType.Function
  }
}

function toSuggestionName(marker: ItemSuggestionMarker): string {
  switch (marker) {
    case ItemSuggestionMarker.Eqv: return 'Eqv'
    case ItemSuggestionMarker.Macro: return 'Macro'
  }
}

interface Item {
  type: TokenType,
  known: boolean,
  next: number
  marker?: ItemSuggestionMarker
  body?: string // for suggestions
}

// Port of https://github.com/1whatleytay/titan/blob/main/src/assembler/lexer.rs
function takeCount(input: string, index: number, take: (c: string) => boolean): number {
  let size = 0

  for (let k = index; k < input.length; k++) {
    if (!take(input[k])) {
      break
    }

    // No UTF8 worries?
    size += 1
  }

  return size
}

const allHard = new Set([
  ':', ';', ',', '{', '}', '+', '-',
  '=', '/', '@', '#', '$', '%', '^', '&',
  '|', '*', '(', ')', '!', '?', '<', '>',
  '~', '[', ']', '\\', '\"', '\''
])

const knownDirectives = new Set([
  'ascii', 'asciiz', 'align', 'space',
  'byte', 'half', 'word', 'float',
  'double', 'text', 'data', 'ktext',
  'kdata', 'extern', 'eqv', 'macro',
  'end_macro'
])

const knownInstructions = new Set([
  'sll', 'srl', 'sra', 'sllv', 'srlv', 'srav', 'jr', 'jalr',
  'mfhi', 'mthi', 'mflo', 'mtlo', 'mult', 'multu', 'div', 'divu',
  'add', 'addu', 'sub', 'subu', 'and', 'or', 'xor', 'nor',
  'sltu', 'slt', 'bltz', 'bgez', 'bltzal', 'bgezal', 'j', 'jal',
  'beq', 'bne', 'blez', 'bgtz', 'addi', 'addiu', 'slti', 'sltiu',
  'andi', 'ori', 'xori', 'lui', 'llo', 'lhi', 'trap', 'syscall',
  'lb', 'lh', 'lw', 'lbu', 'lhu', 'sb', 'sh', 'sw',
  'madd', 'maddu', 'mul', 'msub', 'msubu', 'abs', 'blt', 'bgt',
  'ble', 'bge', 'neg', 'negu', 'not', 'li', 'la', 'move',
  'sge', 'sgt', 'b', 'subi', 'subiu', 'nop', 'bltu', 'bleu',
  'bgtu', 'bgeu', 'sle', 'sgeu', 'sgtu', 'sleu', 'sne', 'seq',
  'beqz', 'bnez'
])

function isWhitespace(c: string): boolean {
  return /\s/.test(c)
}

function takeName(input: string, index: number): number {
  return takeCount(input, index, c => !allHard.has(c) && !isWhitespace(c))
}

function takeSpace(input: string, index: number): number {
  return takeCount(input, index, c => isWhitespace(c))
}

function takeSome(input: string, index: number): number {
  if (index >= input.length) {
    return 0
  }

  const some = (c: string) => allHard.has(c) || isWhitespace(c)
  const isHard = some(input[index])

  return takeCount(input, index + 1, c => some(c) == isHard) + 1
}

function takeStringBody(input: string, index: number, quote: string): number {
  let count = 0

  while (index + count < input.length) {
    const c = input[index + count]

    if (c == quote) {
      break
    } else if (c == '\\') {
      count += 1
    }

    count += 1
  }

  return count
}

function readItem(line: string, index: number, initial: boolean): Item {
  // assert index < line.length

  const start = index + takeSpace(line, index)

  if (start >= line.length) {
    return {
      type: TokenType.Nothing,
      known: false,
      next: start
    }
  }

  const first = line[start]

  switch (first) {
    case '#': {
      const count = takeCount(line, start + 1, c => c != '\n')

      return {
        type: TokenType.Comment,
        known: false,
        next: start + 1 + count
      }
    }

    case '.': {
      const count = takeName(line, start + 1)

      const body = line.substring(start + 1, start + 1 + count)

      let marker: ItemSuggestionMarker | undefined = undefined

      switch (body) {
        case 'eqv':
          marker = ItemSuggestionMarker.Eqv
          break

        case 'macro':
          marker = ItemSuggestionMarker.Macro
          break
      }

      return {
        type: TokenType.Directive,
        known: knownDirectives.has(line.substring(start + 1, start + 1 + count)),
        next: start + 1 + count,
        marker
      }
    }

    case '%': {
      const count = takeName(line, start + 1)

      return {
        type: TokenType.Parameter,
        known: false,
        next: start + 1 + count
      }
    }

    case '$': {
      const count = takeName(line, start + 1)

      return {
        type: TokenType.Register,
        known: false,
        next: start + 1 + count
      }
    }

    case ',':
    case '+':
    case '-':
    case '(':
    case ')':
    case ':':
      return {
        type: TokenType.Hard,
        known: false,
        next: start + 1
      }

    case '\'':
    case '\"': {
      const count = takeStringBody(line, start + 1, first)

      return {
        type: TokenType.Text,
        known: false,
        next: Math.min(start + 1 + count + 1, line.length)
      }
    }

    default: {
      const count = takeName(line, start)

      if (/\d/.test(first)) {
        // digit
        return {
          type: TokenType.Numeric,
          known: false,
          next: start + count
        }
      } else {
        // Check for color for labels
        const nextUp = start + count + takeSpace(line, start + count)

        const body = line.substring(start, start + count)

        if (nextUp < line.length && line[nextUp] == ':') {
          return {
            type: TokenType.Label,
            known: false,
            next: nextUp + 1, // including colon
            body
          }
        }

        // It's an instruction!
        if (initial) {
          return {
            type: TokenType.Instruction,
            known: knownInstructions.has(line.substring(start, start + count)),
            next: start + count
          }
        }

        return {
          type: TokenType.Symbol,
          known: false,
          next: start + count,
          body
        }
      }
    }
  }
}

export function lex(line: string): HighlightResult {
  const tokens: Token[] = []
  const suggestions: MarkedSuggestion[] = []

  let index = 0
  let initial = true

  let marker: ItemSuggestionMarker | undefined = undefined

  while (index < line.length) {
    const item = readItem(line, index, initial)

    // Labels are like, the only token that can be placed before an instruction.
    if (item.type !== TokenType.Label) {
      initial = false
    }

    if (item.body) {
      if (item.type == TokenType.Label) {
        suggestions.push({
          replace: item.body,
          name: 'Label',
          type: SuggestionType.Label,
          index: index + takeSpace(line, index) // start of the line body
        })
      } else if (marker !== undefined) {
        suggestions.push({
          replace: item.body,
          type: toSuggestionType(marker),
          name: toSuggestionName(marker),
          index: index + takeSpace(line, index)
        })
      }
    }

    if (index === item.next) {
      const some = takeSome(line, item.next)

      tokens.push({
        start: index,
        text: line.substring(index, index + some),
        type: TokenType.Nothing,
        color: getStyle(TokenType.Nothing)
      })

      index += some
    } else {
      tokens.push({
        start: index,
        text: line.substring(index, item.next),
        type: item.type,
        color: getStyle(item.type, item.known)
      })

      index = item.next
    }

    marker = item.marker
  }

  return { tokens, suggestions }
}
