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
        { detail: 'Zero Register', label: 'zero' },
        { detail: 'Return Address', label: 'x1' },
        { detail: 'Return Address', label: 'ra' },
        { detail: 'Stack Pointer', label: 'x2' },
        { detail: 'Stack Pointer', label: 'sp' },
        { detail: 'Global Pointer', label: 'x3' },
        { detail: 'Global Pointer', label: 'gp' },
        { detail: 'Thread Pointer', label: 'x4' },
        { detail: 'Thread Pointer', label: 'tp' },
        { detail: 'Temporary Register 0', label: 'x5' },
        { detail: 'Temporary Register 0', label: 't0' },
        { detail: 'Temporary Register 1', label: 'x6' },
        { detail: 'Temporary Register 1', label: 't1' },
        { detail: 'Temporary Register 2', label: 'x7' },
        { detail: 'Temporary Register 2', label: 't2' },
        { detail: 'Frame Pointer / Saved Register 0', label: 'x8' },
        { detail: 'Frame Pointer / Saved Register 0', label: 's0' },
        { detail: 'Saved Register 1', label: 'x9' },
        { detail: 'Saved Register 1', label: 's1' },
        { detail: 'Function Argument 0', label: 'x10' },
        { detail: 'Function Argument 0', label: 'a0' },
        { detail: 'Function Argument 1', label: 'x11' },
        { detail: 'Function Argument 1', label: 'a1' },
        { detail: 'Function Argument 2', label: 'x12' },
        { detail: 'Function Argument 2', label: 'a2' },
        { detail: 'Function Argument 3', label: 'x13' },
        { detail: 'Function Argument 3', label: 'a3' },
        { detail: 'Function Argument 4', label: 'x14' },
        { detail: 'Function Argument 4', label: 'a4' },
        { detail: 'Function Argument 5', label: 'x15' },
        { detail: 'Function Argument 5', label: 'a5' },
        { detail: 'Function Argument 6', label: 'x16' },
        { detail: 'Function Argument 6', label: 'a6' },
        { detail: 'Function Argument 7', label: 'x17' },
        { detail: 'Function Argument 7', label: 'a7' },
        { detail: 'Saved Register 2', label: 'x18' },
        { detail: 'Saved Register 2', label: 's2' },
        { detail: 'Saved Register 3', label: 'x19' },
        { detail: 'Saved Register 3', label: 's3' },
        { detail: 'Saved Register 4', label: 'x20' },
        { detail: 'Saved Register 4', label: 's4' },
        { detail: 'Saved Register 5', label: 'x21' },
        { detail: 'Saved Register 5', label: 's5' },
        { detail: 'Saved Register 6', label: 'x22' },
        { detail: 'Saved Register 6', label: 's6' },
        { detail: 'Saved Register 7', label: 'x23' },
        { detail: 'Saved Register 7', label: 's7' },
        { detail: 'Saved Register 8', label: 'x24' },
        { detail: 'Saved Register 8', label: 's8' },
        { detail: 'Saved Register 9', label: 'x25' },
        { detail: 'Saved Register 9', label: 's9' },
        { detail: 'Saved Register 10', label: 'x26' },
        { detail: 'Saved Register 10', label: 's10' },
        { detail: 'Saved Register 11', label: 'x27' },
        { detail: 'Saved Register 11', label: 's11' },
        { detail: 'Temporary Register 3', label: 'x28' },
        { detail: 'Temporary Register 3', label: 't3' },
        { detail: 'Temporary Register 4', label: 'x29' },
        { detail: 'Temporary Register 4', label: 't4' },
        { detail: 'Temporary Register 5', label: 'x30' },
        { detail: 'Temporary Register 5', label: 't5' },
        { detail: 'Temporary Register 6', label: 'x31' },
        { detail: 'Temporary Register 6', label: 't6' },
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
        { detail: 'Add', label: 'add' },
        { detail: 'Add Immediate', label: 'addi' },
        { detail: 'Negate', label: 'neg' },
        { detail: 'Subtract', label: 'sub' },
        { detail: 'Multiply', label: 'mul' },
        { detail: 'Multiply High', label: 'mulh' },
        { detail: 'Multiply High Unsigned', label: 'mulhu' },
        { detail: 'Multiply High Signed Unsigned', label: 'mulhsu' },
        { detail: 'Divide', label: 'div' },
        { detail: 'Remainder', label: 'rem' },
        { detail: 'And', label: 'and' },
        { detail: 'And Immediate', label: 'andi' },
        { detail: 'Not', label: 'not' },
        { detail: 'Or', label: 'or' },
        { detail: 'Or Immediate', label: 'ori' },
        { detail: 'Xor', label: 'xor' },
        { detail: 'Xor Immediate', label: 'xori' },
        { detail: 'Shift Left Logical', label: 'sll' },
        { detail: 'Shift Left Logical Immediate', label: 'slli' },
        { detail: 'Shift Right Logical', label: 'srl' },
        { detail: 'Shift Right Logical Immediate', label: 'srli' },
        { detail: 'Shift Right Arithmetic', label: 'sra' },
        { detail: 'Shift Right Arithmetic Immediate', label: 'srai' },
        { detail: 'Load Immediate', label: 'li' },
        { detail: 'Load Upper Immediate', label: 'lui' },
        { detail: 'Add Upper Immediate to PC', label: 'auipc' },
        { detail: 'Load Word', label: 'lw' },
        { detail: 'Load Half', label: 'lh' },
        { detail: 'Load Half Unsigned', label: 'lhu' },
        { detail: 'Load Byte', label: 'lb' },
        { detail: 'Load Byte Unsigned', label: 'lbu' },
        { detail: 'Load Symbol Address', label: 'la' },
        { detail: 'Store Word', label: 'sw' },
        { detail: 'Store Half', label: 'sh' },
        { detail: 'Store Byte', label: 'sb' },
        { detail: 'Jump', label: 'j' },
        { detail: 'Jump and Link', label: 'jal' },
        { detail: 'Jump and Link Register', label: 'jalr' },
        { detail: 'Call Function', label: 'call' },
        { detail: 'Return from Function', label: 'ret' },
        { detail: 'Branch Equal', label: 'beq' },
        { detail: 'Branch Equal Zero', label: 'beqz' },
        { detail: 'Branch Not Equal', label: 'bne' },
        { detail: 'Branch Not Equal Zero', label: 'bnez' },
        { detail: 'Branch Less Than', label: 'blt' },
        { detail: 'Branch Less Than Unsigned', label: 'bltu' },
        { detail: 'Branch Less Than Zero', label: 'bltz' },
        { detail: 'Branch Greater Than', label: 'bgt' },
        { detail: 'Branch Greater Than Unsigned', label: 'bgtu' },
        { detail: 'Branch Greater Than Zero', label: 'bgtz' },
        { detail: 'Branch Less or Equal', label: 'ble' },
        { detail: 'Branch Less or Equal Unsigned', label: 'bleu' },
        { detail: 'Branch Less or Equal Zero', label: 'blez' },
        { detail: 'Branch Greater or Equal', label: 'bge' },
        { detail: 'Branch Greater or Equal Unsigned', label: 'bgeu' },
        { detail: 'Branch Greater or Equal Zero', label: 'bgez' },
        { detail: 'Set Less Than', label: 'slt' },
        { detail: 'Set Less Than Immediate', label: 'slti' },
        { detail: 'Set Less Than Unsigned', label: 'sltu' },
        { detail: 'Set Less Than Immediate Unsigned', label: 'sltiu' },
        { detail: 'Set Equal Zero', label: 'seqz' },
        { detail: 'Set Not Equal Zero', label: 'snez' },
        { detail: 'Set Less Than Zero', label: 'sltz' },
        { detail: 'Set Greater Than Zero', label: 'sgtz' },
      ].map((x) => ({ ...x, type: 'instruction' })),
      ...labels,
    ],
  }
}
