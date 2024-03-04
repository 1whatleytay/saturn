/*
import * as backend from './wasm/saturn_wasm'
import { ExportRegionsOptions } from '../settings'
import {
  AssembledRegions,
  AssemblerResult,
  BinaryResult, BitmapConfig,
  DisassembleResult,
  HexBinaryResult,
  InstructionDetails, InstructionLine, LastDisplay
} from './mips'

// Runner/Execution State (Automatically Freed with the Worker Memory)
const runner = new backend.Runner();

interface AssembleRegionsInput {
  text: string
  path: string | null
  options: ExportRegionsOptions
}

function assembleRegions({ text, options }: AssembleRegionsInput): HexBinaryResult {
  const [regions, result] = backend.assemble_regions(text, options) as [
    AssembledRegions | null,
    AssemblerResult
  ]

  return {
    regions,
    result
  }
}

interface AssembleTextInput {
  text: string
  path: string | null
}

function assembleText({ text }: AssembleTextInput): AssemblerResult {
  return backend.assemble_text(text) as AssemblerResult
}

interface AssembleBinaryInput {
  text: string
  path: string | null
}

function assembleBinary({ text }: AssembleBinaryInput): BinaryResult {
  return backend.assemble_binary(text) as BinaryResult
}

interface DecodeInstructionInput {
  pc: number
  instruction: number
}

function decodeInstruction({ pc, instruction }: DecodeInstructionInput): InstructionDetails | null {
  return backend.decode_instruction(pc, instruction) as InstructionDetails | null
}

interface DisassembleInput {
  named: string | null
  bytes: Uint8Array
}

function disassemble({ named, bytes }: DisassembleInput): DisassembleResult {
  return backend.disassemble(named ?? undefined, bytes) as DisassembleResult
}

interface DetailedDisassembleInput {
  bytes: Uint8Array
}

function detailedDisassemble({ bytes }: DetailedDisassembleInput): InstructionLine[] {
  return backend.detailed_disassemble(bytes) as InstructionLine[]
}

interface ConfigureDisplayInput {
  config: BitmapConfig
}

function configureDisplay({ config }: ConfigureDisplayInput) {
  runner.configure_display(config.address, config.width, config.height)
}

function lastDisplay(): LastDisplay {
  return runner.last_display()
}

interface ConfigureElfInput {
  bytes: Uint8Array
  timeTravel: boolean
}

function configureElf({ bytes, timeTravel }: ConfigureElfInput) {
  runner.configure_elf(bytes, timeTravel)
}

interface ConfigureAsmInput {
  text: string
  timeTravel: boolean
}

function configureAsm({ text, timeTravel }: ConfigureAsmInput) {
  runner.configure_asm(text, timeTravel)
}

async function dispatchOp(op: string, data: any): Promise<any> {
  switch (op) {
    case 'assemble_regions': return assembleRegions(data as AssembleRegionsInput)
    case 'assemble_text': return assembleText(data as AssembleTextInput)
    case 'assemble_binary': return assembleBinary(data as AssembleBinaryInput)
    case 'decode_instruction': return decodeInstruction(data as DecodeInstructionInput)
    case 'disassemble': return disassemble(data as DisassembleInput)
    case 'detailed_disassemble': return detailedDisassemble(data as DetailedDisassembleInput)
    case 'configure_display': return configureDisplay(data as ConfigureDisplayInput)
    case 'last_display': return lastDisplay()
    case 'configure_elf': return configureElf(data as ConfigureElfInput)
    case 'configure_asm': return configureAsm(data as ConfigureAsmInput)
    default: throw new Error(`Unknown worker op ${op}`)
  }
}

async function handleMessage(event: MessageEvent) {
  const { requestId, op, data } = event.data

  try {
    const value = await dispatchOp(op, data)

    postMessage({
      requestId,
      data: value
    })
  } catch (error) {
    postMessage({
      requestId,
      error
    })
  }
}

onmessage = handleMessage
*/

export { }