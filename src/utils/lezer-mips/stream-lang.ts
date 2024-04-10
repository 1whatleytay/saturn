import {
  LanguageSupport,
  StreamLanguage,
  StringStream,
} from '@codemirror/language'
import { TokenType } from '../languages/language'
import { myCompletions } from './autocomplete'

enum ItemSuggestionMarker {
  Macro,
  Eqv,
}

interface Item {
  type: TokenType
  known: boolean
  marker?: ItemSuggestionMarker
  body?: string // for suggestions
}

// Port of https://github.com/1whatleytay/titan/blob/main/src/assembler/lexer.rs
const allHard = new Set([
  ':',
  ';',
  ',',
  '{',
  '}',
  '+',
  '-',
  '=',
  '/',
  '@',
  '#',
  '$',
  '%',
  '^',
  '&',
  '|',
  '*',
  '(',
  ')',
  '!',
  '?',
  '<',
  '>',
  '~',
  '[',
  ']',
  '\\',
  '"',
  "'",
])

const knownDirectives = new Set([
  'ascii',
  'asciiz',
  'align',
  'space',
  'byte',
  'half',
  'word',
  'float',
  'double',
  'entry',
  'text',
  'data',
  'ktext',
  'kdata',
  'extern',
  'eqv',
  'macro',
  'end_macro',
  'include',
])

const knownInstructions = new Set([
  'sll',
  'srl',
  'sra',
  'sllv',
  'srlv',
  'srav',
  'jr',
  'jalr',
  'mfhi',
  'mthi',
  'mflo',
  'mtlo',
  'mult',
  'multu',
  'div',
  'divu',
  'add',
  'addu',
  'sub',
  'subu',
  'and',
  'or',
  'xor',
  'nor',
  'sltu',
  'slt',
  'bltz',
  'bgez',
  'bltzal',
  'bgezal',
  'j',
  'jal',
  'beq',
  'bne',
  'blez',
  'bgtz',
  'addi',
  'addiu',
  'slti',
  'sltiu',
  'andi',
  'ori',
  'xori',
  'lui',
  'llo',
  'lhi',
  'trap',
  'syscall',
  'lb',
  'lh',
  'lw',
  'lbu',
  'lhu',
  'sb',
  'sh',
  'sw',
  'madd',
  'maddu',
  'mul',
  'msub',
  'msubu',
  'abs',
  'blt',
  'bgt',
  'ble',
  'bge',
  'neg',
  'negu',
  'not',
  'li',
  'la',
  'move',
  'sge',
  'sgt',
  'b',
  'subi',
  'subiu',
  'nop',
  'bltu',
  'bleu',
  'bgtu',
  'bgeu',
  'sle',
  'sgeu',
  'sgtu',
  'sleu',
  'sne',
  'seq',
  'beqz',
  'bnez',
])

function isWhitespace(c: string): boolean {
  return /\s/.test(c)
}

function takeName(input: StringStream): string {
  let out = ''
  let c
  do {
    c = input.eat((c) => !allHard.has(c) && !isWhitespace(c))!
    out += c
  } while (c)
  return out
}

function spaceThenColon(input: StringStream) {
  return input.match(/^\s*:/, false)!
}
function takeSpace(input: StringStream): string {
  return input.eat(isWhitespace)!
}

function takeStringBody(input: StringStream, quote: string): void {
  while (true) {
    const c = input.next()!

    if (c == quote) {
      break
    } else if (c == '\\') {
      input.next
    }
  }
}

function readItem(line: StringStream, initial: boolean): Item | undefined {
  initial ||= line.sol()
  line.eatWhile(isWhitespace)

  const first = line.next()!

  switch (first) {
    case '#': {
      line.eatWhile((c) => c != '\n')

      return {
        type: TokenType.Comment,
        known: false,
      }
    }

    case '.': {
      const body = takeName(line)

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
        known: knownDirectives.has(body),
        marker,
      }
    }

    case '%': {
      takeName(line)

      return {
        type: TokenType.Parameter,
        known: false,
        // next: start + 1 + count,
      }
    }

    case '$': {
      takeName(line)

      return {
        type: TokenType.Register,
        known: false,
      }
    }

    case '(':
      return {
        type: TokenType.BracketOpen,
        known: false,
      }

    case ')':
      return {
        type: TokenType.BracketClose,
        known: false,
      }

    case ',':
    case '+':
    case '-':
    case ':':
      return {
        type: TokenType.Hard,
        known: false,
      }

    case "'":
    case '"': {
      takeStringBody(line, first)

      return {
        type: TokenType.Text,
        known: false,
      }
    }

    case undefined:
      return undefined

    default: {
      line.backUp(1)

      const body = takeName(line)

      if (/\d/.test(first)) {
        // digit
        return {
          type: TokenType.Numeric,
          known: false,
        }
      } else {
        // Check for color for labels
        const nextUp = spaceThenColon(line)

        if (nextUp) {
          takeSpace(line)
          line.next()

          return {
            type: TokenType.Label,
            known: false,
            body,
          }
        }

        // It's an instruction!
        if (initial) {
          return {
            type: TokenType.Instruction,
            known: knownInstructions.has(body),
          }
        }

        return {
          type: TokenType.Symbol,
          known: false,
          body,
        }
      }
    }
  }
}

function typeToTag(type: TokenType): string {
  const x = {
    [TokenType.Comment]: 'lineComment',
    [TokenType.Hard]: 'separator',
    [TokenType.BracketOpen]: 'paren',
    [TokenType.BracketClose]: 'paren',
    [TokenType.Label]: 'attributeName',
    [TokenType.Directive]: 'macroName',
    [TokenType.Parameter]: 'typeName',
    [TokenType.Instruction]: 'keyword',
    [TokenType.Register]: 'typeName',
    [TokenType.Numeric]: 'number',
    [TokenType.Symbol]: 'variableName',
    [TokenType.Text]: 'string',
    [TokenType.Nothing]: 'invalid',
  }
  return x[type]
}
const sl = StreamLanguage.define({
  name: 'mips',
  startState() {
    return {
      initial: true,
    }
  },
  token(stream, state) {
    const item = readItem(stream, state.initial)
    if (!item) {
      return null
    }
    state.initial = item.type === TokenType.Label
    return typeToTag(item.type)
  },
  languageData: {
    commentTokens: { line: '#' },
    autocomplete: myCompletions,
  },
})

export const lang = new LanguageSupport(sl)
