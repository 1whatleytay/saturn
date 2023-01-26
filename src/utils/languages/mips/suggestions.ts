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

export function mergeResults(input: FuseResult<Suggestion>[][]): FuseResult<Suggestion>[] {
  const values = input.map(x => x.reverse()) // we want to pop

  const result = [] as FuseResult<Suggestion>[]

  while (true) {
    let next = null as FuseResult<Suggestion>[] | null

    for (const value of values) {
      if (!value.length) {
        continue
      }

      if (!next || (next[next.length - 1].score ?? 0) < (value[value.length - 1].score ?? 0)) {
        next = value
      }
    }

    if (!next) {
      return result
    }

    result.push(next.pop()!)
  }
}

const instructionParts = [
  { name: 'sll', replace: 'sll' },
  { name: 'srl', replace: 'srl' },
  { name: 'sra', replace: 'sra' },
  { name: 'sllv', replace: 'sllv' },
  { name: 'srlv', replace: 'srlv' },
  { name: 'srav', replace: 'srav' },
  { name: 'jr', replace: 'jr' },
  { name: 'jalr', replace: 'jalr' },
  { name: 'mfhi', replace: 'mfhi' },
  { name: 'mthi', replace: 'mthi' },
  { name: 'mflo', replace: 'mflo' },
  { name: 'mtlo', replace: 'mtlo' },
  { name: 'mult', replace: 'mult' },
  { name: 'multu', replace: 'multu' },
  { name: 'div', replace: 'div' },
  { name: 'divu', replace: 'divu' },
  { name: 'add', replace: 'add' },
  { name: 'addu', replace: 'addu' },
  { name: 'sub', replace: 'sub' },
  { name: 'subu', replace: 'subu' },
  { name: 'and', replace: 'and' },
  { name: 'or', replace: 'or' },
  { name: 'xor', replace: 'xor' },
  { name: 'nor', replace: 'nor' },
  { name: 'sltu', replace: 'sltu' },
  { name: 'slt', replace: 'slt' },
  { name: 'bltz', replace: 'bltz' },
  { name: 'bgez', replace: 'bgez' },
  { name: 'bltzal', replace: 'bltzal' },
  { name: 'bgezal', replace: 'bgezal' },
  { name: 'j', replace: 'j' },
  { name: 'jal', replace: 'jal' },
  { name: 'beq', replace: 'beq' },
  { name: 'bne', replace: 'bne' },
  { name: 'blez', replace: 'blez' },
  { name: 'bgtz', replace: 'bgtz' },
  { name: 'addi', replace: 'addi' },
  { name: 'addiu', replace: 'addiu' },
  { name: 'slti', replace: 'slti' },
  { name: 'sltiu', replace: 'sltiu' },
  { name: 'andi', replace: 'andi' },
  { name: 'ori', replace: 'ori' },
  { name: 'xori', replace: 'xori' },
  { name: 'lui', replace: 'lui' },
  { name: 'llo', replace: 'llo' },
  { name: 'lhi', replace: 'lhi' },
  { name: 'trap', replace: 'trap' },
  { name: 'syscall', replace: 'syscall' },
  { name: 'lb', replace: 'lb' },
  { name: 'lh', replace: 'lh' },
  { name: 'lw', replace: 'lw' },
  { name: 'lbu', replace: 'lbu' },
  { name: 'lhu', replace: 'lhu' },
  { name: 'sb', replace: 'sb' },
  { name: 'sh', replace: 'sh' },
  { name: 'sw', replace: 'sw' },
  { name: 'madd', replace: 'madd' },
  { name: 'maddu', replace: 'maddu' },
  { name: 'mul', replace: 'mul' },
  { name: 'msub', replace: 'msub' },
  { name: 'msubu', replace: 'msubu' },
  { name: 'abs', replace: 'abs' },
  { name: 'blt', replace: 'blt' },
  { name: 'bgt', replace: 'bgt' },
  { name: 'ble', replace: 'ble' },
  { name: 'bge', replace: 'bge' },
  { name: 'neg', replace: 'neg' },
  { name: 'negu', replace: 'negu' },
  { name: 'not', replace: 'not' },
  { name: 'li', replace: 'li' },
  { name: 'la', replace: 'la' },
  { name: 'move', replace: 'move' },
  { name: 'sge', replace: 'sge' },
  { name: 'sgt', replace: 'sgt' },
  { name: 'b', replace: 'b' },
  { name: 'subi', replace: 'subi' },
  { name: 'subiu', replace: 'subiu' },
].map(x => ({ ...x, type: SuggestionType.Instruction })) as Suggestion[]

const registerParts = [
  { name: '$zero', replace: '$zero' },
  { name: '$at', replace: '$at' },
  { name: '$v0', replace: '$v0' },
  { name: '$v1', replace: '$v1' },
  { name: '$a0', replace: '$a0' },
  { name: '$a1', replace: '$a1' },
  { name: '$a2', replace: '$a2' },
  { name: '$a3', replace: '$a3' },
  { name: '$t0', replace: '$t0' },
  { name: '$t1', replace: '$t1' },
  { name: '$t2', replace: '$t2' },
  { name: '$t3', replace: '$t3' },
  { name: '$t4', replace: '$t4' },
  { name: '$t5', replace: '$t5' },
  { name: '$t6', replace: '$t6' },
  { name: '$t7', replace: '$t7' },
  { name: '$s0', replace: '$s0' },
  { name: '$s1', replace: '$s1' },
  { name: '$s2', replace: '$s2' },
  { name: '$s3', replace: '$s3' },
  { name: '$s4', replace: '$s4' },
  { name: '$s5', replace: '$s5' },
  { name: '$s6', replace: '$s6' },
  { name: '$s7', replace: '$s7' },
  { name: '$t8', replace: '$t8' },
  { name: '$t9', replace: '$t9' },
  { name: '$k0', replace: '$k0' },
  { name: '$k1', replace: '$k1' },
  { name: '$gp', replace: '$gp' },
  { name: '$sp', replace: '$sp' },
  { name: '$fp', replace: '$fp' },
  { name: '$ra', replace: '$ra' },
].map(x => ({ ...x, type: SuggestionType.Register })) as Suggestion[]

const directiveParts = [
  { name: '.ascii', replace: '.ascii' },
  { name: '.asciiz', replace: '.asciiz' },
  { name: '.align', replace: '.align' },
  { name: '.space', replace: '.space' },
  { name: '.byte', replace: '.byte' },
  { name: '.half', replace: '.half' },
  { name: '.word', replace: '.word' },
  { name: '.float', replace: '.float' },
  { name: '.double', replace: '.double' },
  { name: '.text', replace: '.text' },
  { name: '.data', replace: '.data' },
  { name: '.ktext', replace: '.ktext' },
  { name: '.extern', replace: '.extern' },
  { name: '.eqv', replace: '.eqv' },
  { name: '.macro', replace: '.macro' },
  { name: '.end_macro', replace: '.end_macro' },
].map(x => ({ ...x, type: SuggestionType.Directive })) as Suggestion[]

const fuseOptions = {
  keys: ['replace'],
  includeScore: true,
  includeMatches: true
}

export const instructions = new Fuse(instructionParts, fuseOptions)
export const registers = new Fuse(registerParts, fuseOptions)
export const directives = new Fuse(directiveParts, fuseOptions)
export const lineSuggestions = new Fuse(instructionParts.concat(directiveParts), fuseOptions)
