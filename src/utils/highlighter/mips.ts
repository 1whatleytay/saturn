import { Highlighter, style, Token } from './highlighter'

interface Item {
  color: string,
  next: number
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
  'extern'
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
  'sge', 'sgt', 'b', 'subi', 'subiu',
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
      color: style.nothing,
      next: start
    }
  }

  const first = line[start]

  switch (first) {
    case '#': {
      const count = takeCount(line, start + 1, c => c != '\n')

      return {
        color: style.comment,
        next: start + 1 + count
      }
    }

    case '.': {
      const count = takeName(line, start + 1)

      const parts = [style.directive]

      if (knownDirectives.has(line.substring(start + 1, start + 1 + count))) {
        parts.push(style.known)
      }

      return {
        color: parts.join(' '),
        next: start + 1 + count
      }
    }

    case '%': {
      const count = takeName(line, start + 1)

      return {
        color: style.directive,
        next: start + 1 + count
      }
    }

    case '$': {
      const count = takeName(line, start + 1)

      return {
        color: style.register,
        next: start + 1 + count
      }
    }

    case ',':
    case '(':
    case ')':
    case ':':
      return {
        color: style.comma,
        next: start + 1
      }

    case '\'':
    case '\"': {
      const count = takeStringBody(line, start + 1, first)

      return {
        color: style.text,
        next: Math.min(start + 1 + count + 1, line.length)
      }
    }

    default: {
      const count = takeName(line, start)

      if (/\d/.test(first)) {
        // digit
        return {
          color: style.numeric,
          next: start + count
        }
      } else {
        if (initial) {
          // First lone symbol on a line *should* be an instruction
          const parts = [style.instruction]

          if (knownInstructions.has(line.substring(start, start + count))) {
            parts.push(style.known)
          }

          return {
            color: parts.join(' '),
            next: start + count
          }
        }

        return {
          color: style.symbol,
          next: start + count
        }
      }
    }
  }
}

export class MipsHighlighter implements Highlighter {
  highlight(line: string): Token[] {
    const result = []

    let index = 0
    let initial = true

    while (index < line.length) {
      const { color, next } = readItem(line, index, initial)
      initial = false

      if (index === next) {
        const some = takeSome(line, next)

        result.push({
          text: line.substring(index, index + some),
          color: style.nothing
        })

        index += some
      } else {
        result.push({ text: line.substring(index, next), color })

        index = next
      }
    }

    return result
  }
}
