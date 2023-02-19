import { Suggestion, SuggestionMatch, SuggestionType } from '../suggestions'
import Fuse from 'fuse.js'
import FuseResult = Fuse.FuseResult

export function toSuggestionMatches(results: FuseResult<Suggestion>[]): SuggestionMatch[] {
  return results.map(x => {
    let range = undefined

    if (x.matches && x.matches.length > 0) {
      const match = x.matches[0]

      const first = match.indices[0]

      range = {
        start: first[0],
        end: first[1]
      }
    }

    return {
      ...x.item,
      range
    } as SuggestionMatch
  })
}

export function mergeResults(... input: FuseResult<Suggestion>[][]): FuseResult<Suggestion>[] {
  const values = input.map(x => x.reverse()) // we want to pop

  const result = [] as FuseResult<Suggestion>[]

  while (true) {
    let next = null as FuseResult<Suggestion>[] | null

    for (const value of values) {
      if (!value.length) {
        continue
      }

      if (!next || (next[next.length - 1].score ?? 0) > (value[value.length - 1].score ?? 0)) {
        next = value
      }
    }

    if (!next) {
      return result
    }

    result.push(next.pop()!)
  }
}

export const registersSet = new Set([
  '$zero', '$at', '$v0', '$v1',
  '$a0', '$a1', '$a2', '$a3',
  '$t0', '$t1', '$t2', '$t3',
  '$t4', '$t5', '$t6', '$t7',
  '$s0', '$s1', '$s2', '$s3',
  '$s4', '$s5', '$s6', '$s7',
  '$t8', '$t9', '$k0', '$k1',
  '$gp', '$sp', '$fp', '$ra',
])

const instructionParts = [
  { name: 'Shift Left', replace: 'sll' },
  { name: 'Shift Right', replace: 'srl' },
  { name: 'Shift Right Signed', replace: 'sra' },
  { name: 'Shift Left Reg', replace: 'sllv' },
  { name: 'Shift Right Reg', replace: 'srlv' },
  { name: 'Shift Right Signed Reg', replace: 'srav' },
  { name: 'Jump Register', replace: 'jr' },
  { name: 'Jump and Link Register', replace: 'jalr' },
  { name: 'Move From HI', replace: 'mfhi' },
  { name: 'Move To HI', replace: 'mthi' },
  { name: 'Move From LO', replace: 'mflo' },
  { name: 'Move To LO', replace: 'mtlo' },
  { name: 'Multiply', replace: 'mult' },
  { name: 'Multiply Unsigned', replace: 'multu' },
  { name: 'Divide', replace: 'div' },
  { name: 'Divide Unsigned', replace: 'divu' },
  { name: 'Add', replace: 'add' },
  { name: 'Add Unsigned', replace: 'addu' },
  { name: 'Subtract', replace: 'sub' },
  { name: 'Subtract Unsigned', replace: 'subu' },
  { name: 'Bitwise AND', replace: 'and' },
  { name: 'Bitwise OR', replace: 'or' },
  { name: 'Bitwise XOR', replace: 'xor' },
  { name: 'Bitwise NOR', replace: 'nor' },
  { name: 'Set Less Than Unsigned', replace: 'sltu' },
  { name: 'Set Less Than', replace: 'slt' },
  { name: 'Branch < 0', replace: 'bltz' },
  { name: 'Branch >= 0', replace: 'bgez' },
  { name: 'Branch < 0 and Link', replace: 'bltzal' },
  { name: 'Branch >= 0 and Link', replace: 'bgezal' },
  { name: 'Jump', replace: 'j' },
  { name: 'Jump and Link', replace: 'jal' },
  { name: 'Branch Equal', replace: 'beq' },
  { name: 'Branch Not Equal', replace: 'bne' },
  { name: 'Branch <= 0', replace: 'blez' },
  { name: 'Branch > 0', replace: 'bgtz' },
  { name: 'Add Immediate', replace: 'addi' },
  { name: 'Add Immediate Unsigned', replace: 'addiu' },
  { name: 'Set Less Than Immediate', replace: 'slti' },
  { name: 'Set Less Than Immediate Unsigned', replace: 'sltiu' },
  { name: 'Bitwise AND Immediate', replace: 'andi' },
  { name: 'Bitwise OR Immediate', replace: 'ori' },
  { name: 'Bitwise XOR Immediate', replace: 'xori' },
  { name: 'Load Upper Immediate', replace: 'lui' },
  { name: 'Load LO', replace: 'llo' },
  { name: 'Load HI', replace: 'lhi' },
  { name: 'Trap Exeption', replace: 'trap' },
  { name: 'System Call', replace: 'syscall' },
  { name: 'Load Byte', replace: 'lb' },
  { name: 'Load Half', replace: 'lh' },
  { name: 'Load Word', replace: 'lw' },
  { name: 'Load Byte Unsigned', replace: 'lbu' },
  { name: 'Load Half Unsigned', replace: 'lhu' },
  { name: 'Store Byte', replace: 'sb' },
  { name: 'Store Half', replace: 'sh' },
  { name: 'Store Word', replace: 'sw' },
  { name: 'Multiply and Add', replace: 'madd' },
  { name: 'Multiply and Add Unsigned', replace: 'maddu' },
  { name: 'Multiply Registers', replace: 'mul' },
  { name: 'Multiply and Subtract', replace: 'msub' },
  { name: 'Multiply and Subtract Unsigned', replace: 'msubu' },
  { name: 'Absolute Value', replace: 'abs' },
  { name: 'Branch Less Than', replace: 'blt' },
  { name: 'Branch Greater Than', replace: 'bgt' },
  { name: 'Branch Less or Equal', replace: 'ble' },
  { name: 'Branch Greater or Equal', replace: 'bge' },
  { name: 'Negate Value', replace: 'neg' },
  { name: 'Negate Unsigned Value', replace: 'negu' },
  { name: 'Bitwise NOT', replace: 'not' },
  { name: 'Load Immediate', replace: 'li' },
  { name: 'Load Address', replace: 'la' },
  { name: 'Move Registers', replace: 'move' },
  { name: 'Set Greater or Equal', replace: 'sge' },
  { name: 'Set Greater Than', replace: 'sgt' },
  { name: 'Branch', replace: 'b' },
  { name: 'Subtract Immediate', replace: 'subi' },
  { name: 'Subtract Immediate Unsigned', replace: 'subiu' },
].map(x => ({ ...x, type: SuggestionType.Instruction })) as Suggestion[]

const registerParts = [
  { name: 'Zero Register', replace: '$zero' },
  { name: 'Assembler Temporary', replace: '$at' },
  { name: 'Value 0', replace: '$v0' },
  { name: 'Value 1', replace: '$v1' },
  { name: 'Parameter 0', replace: '$a0' },
  { name: 'Parameter 1', replace: '$a1' },
  { name: 'Parameter 2', replace: '$a2' },
  { name: 'Parameter 3', replace: '$a3' },
  { name: 'Temporary 0', replace: '$t0' },
  { name: 'Temporary 1', replace: '$t1' },
  { name: 'Temporary 2', replace: '$t2' },
  { name: 'Temporary 3', replace: '$t3' },
  { name: 'Temporary 4', replace: '$t4' },
  { name: 'Temporary 5', replace: '$t5' },
  { name: 'Temporary 6', replace: '$t6' },
  { name: 'Temporary 7', replace: '$t7' },
  { name: 'Saved 0', replace: '$s0' },
  { name: 'Saved 1', replace: '$s1' },
  { name: 'Saved 2', replace: '$s2' },
  { name: 'Saved 3', replace: '$s3' },
  { name: 'Saved 4', replace: '$s4' },
  { name: 'Saved 5', replace: '$s5' },
  { name: 'Saved 6', replace: '$s6' },
  { name: 'Saved 7', replace: '$s7' },
  { name: 'Temporary 8', replace: '$t8' },
  { name: 'Temporary 9', replace: '$t9' },
  { name: 'Kernel 0', replace: '$k0' },
  { name: 'Kernel 1', replace: '$k1' },
  { name: 'General Pointer', replace: '$gp' },
  { name: 'Stack Pointer', replace: '$sp' },
  { name: 'Frame Pointer', replace: '$fp' },
  { name: 'Return Address', replace: '$ra' },
].map(x => ({ ...x, type: SuggestionType.Register })) as Suggestion[]

const directiveParts = [
  { name: 'Ascii Text', replace: '.ascii' },
  { name: 'Ascii Zero Terminated', replace: '.asciiz' },
  { name: 'Align Bytes', replace: '.align' },
  { name: 'Space Bytes', replace: '.space' },
  { name: 'Byte Literals', replace: '.byte' },
  { name: 'Half Literals', replace: '.half' },
  { name: 'Word Literals', replace: '.word' },
  { name: 'Float Literals', replace: '.float' },
  { name: 'Double Literals', replace: '.double' },
  { name: 'Text Section', replace: '.text' },
  { name: 'Data Section', replace: '.data' },
  { name: 'Kernel Text Section', replace: '.ktext' },
  { name: 'Kernel Data Section', replace: '.kdata' },
  { name: 'Extern Symbol', replace: '.extern' },
  { name: 'Define Token', replace: '.eqv' },
  { name: 'Define Macro', replace: '.macro' },
  { name: 'End Macro', replace: '.end_macro' },
].map(x => ({ ...x, type: SuggestionType.Directive })) as Suggestion[]

export const fuseOptions = {
  keys: ['replace'],
  includeScore: true,
  includeMatches: true
}

export const instructions = new Fuse(instructionParts, fuseOptions)
export const registers = new Fuse(registerParts, fuseOptions)
export const directives = new Fuse(directiveParts, fuseOptions)
