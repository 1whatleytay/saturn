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
  while (true) {
    c = input.eat((c) => !allHard.has(c) && !isWhitespace(c))!
    if (c) out += c
    else {
      break
    }
  }
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
      input.next()
    }
  }
}

function readItem(line: StringStream, initial: boolean): TokenType | undefined {
  initial ||= line.sol()
  line.eatWhile(isWhitespace)

  const first = line.peek()!

  switch (first) {
    case '#': {
      line.next()
      line.skipToEnd()
      // line.eatWhile((c) => c != '\n')

      return TokenType.Comment
    }

    case '.': {
      line.next()
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

      return TokenType.Directive
    }

    case '%': {
      line.next()
      takeName(line)

      return TokenType.Parameter
    }

    case '$': {
      line.next()
      takeName(line)

      return TokenType.Register
    }

    case '(':
      line.next()
      return TokenType.BracketOpen

    case ')':
      line.next()
      return TokenType.BracketClose

    case ',':
    case '+':
    case '-':
    case ':':
      line.next()
      return TokenType.Hard

    case "'":
    case '"': {
      line.next()
      takeStringBody(line, first)

      return TokenType.Text
    }

    case undefined:
      return undefined

    default: {
      const body = takeName(line)

      if (/\d/.test(first)) {
        // digit
        return TokenType.Numeric
      } else {
        // Check for color for labels
        const nextUp = spaceThenColon(line)

        if (nextUp) {
          takeSpace(line)
          line.next()

          return TokenType.Label
        }

        // It's an instruction!
        if (initial) {
          return TokenType.Instruction
        }

        return TokenType.Symbol
      }
    }
  }
}

const typeToTag = {
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
    state.initial = item === TokenType.Label
    return typeToTag[item]
  },
  languageData: {
    commentTokens: { line: '#' },
    autocomplete: myCompletions,
  },
})

export const lang = new LanguageSupport(sl)
