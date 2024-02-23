import {
  AssemblerResult,
  BinaryResult,
  BitmapConfig, DisassembleResult,
  ExecutionProfile,
  HexBinaryResult, InstructionDetails, InstructionLine, LastDisplay,
  MipsBackend,
  MipsExecution
} from './mips'
import { ExportRegionsOptions } from '../settings'

export class WasmBackend implements MipsBackend {
  assembleRegions(text: string, path: string | null, options: ExportRegionsOptions): Promise<HexBinaryResult> {
    throw new Error()
  }

  assembleText(text: string, path: string | null): Promise<AssemblerResult> {
    throw new Error()
  }

  assembleWithBinary(text: string, path: string | null): Promise<BinaryResult> {
    throw new Error()
  }

  configureDisplay(config: BitmapConfig): Promise<void> {
    throw new Error()
  }

  createExecution(text: string, path: string | null, timeTravel: boolean, profile: ExecutionProfile): Promise<MipsExecution> {
    throw new Error()
  }

  decodeInstruction(pc: number, instruction: number): Promise<InstructionDetails | null> {
    throw new Error()
  }

  disassembleElf(named: string, elf: ArrayBuffer): Promise<DisassembleResult> {
    throw new Error()
  }

  disassemblyDetails(bytes: ArrayBuffer): Promise<InstructionLine[]> {
    throw new Error()
  }

  lastDisplay(): Promise<LastDisplay> {
    throw new Error()
  }
}