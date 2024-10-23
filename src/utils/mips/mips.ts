import { ExportRegionsOptions } from '../settings'
import { MidiNote } from '../midi'

export interface ElfExecutionProfile {
  kind: 'elf'
  elf: string // not sure what to do here
  breakpoints: Record<number, number>
}

// should only be set after building
export interface AssemblyExecutionProfile {
  kind: 'asm'
}

export type ExecutionProfile = ElfExecutionProfile | AssemblyExecutionProfile

export interface DisassembleResult {
  error: string | null
  lines: string[]
  breakpoints: Record<number, number>
}

export interface LineMarker {
  line: number
  offset: number
}

export interface Breakpoint {
  line: number
  pcs: number[]
}

export interface AssemblerSuccess {
  breakpoints: Breakpoint[]
}

export interface AssemblerError {
  marker: LineMarker | null
  body: string | null
  message: string
}

export type AssemblerResultSuccess = AssemblerSuccess & { status: 'Success' }
export type AssemblerResultError = AssemblerError & { status: 'Error' }

export type AssemblerResult = AssemblerResultSuccess | AssemblerResultError

export interface BinaryResult {
  binary: Uint8Array | null
  result: AssemblerResult
}

export interface BitmapConfig {
  width: number
  height: number
  address: number
}

export enum ExecutionModeType {
  Running = 'Running',
  Invalid = 'Invalid',
  Paused = 'Paused',
  Stopped = 'Stopped',
  Breakpoint = 'Breakpoint',
  Finished = 'Finished',
}

export interface ExecutionModeInvalid {
  type: ExecutionModeType.Invalid
  message: string
}

export interface ExecutionModeFinished {
  type: ExecutionModeType.Finished
  pc: number
  code: number | null
}

type ExecutionModeOther =
  | ExecutionModeType.Running
  | ExecutionModeType.Breakpoint
  | ExecutionModeType.Paused

export type ExecutionMode =
  | ExecutionModeInvalid
  | ExecutionModeFinished
  | { type: ExecutionModeOther }

export interface Registers {
  pc: number
  line: number[]
  lo: number
  hi: number
}

export interface ExecutionResult {
  mode: ExecutionMode
  registers: Registers
}

export interface LastDisplay {
  address: number
  width: number
  height: number
  data: number[] | null
}

export class Breakpoints {
  public maxLine: number
  public lineToPc: Map<number, number[]>
  public pcToGroup: Map<number, Breakpoint>

  findNextPc(line: number): number[] {
    while (line <= this.maxLine) {
      const value = this.lineToPc.get(line)

      if (value !== undefined) {
        return value
      }

      line += 1
    }

    return []
  }

  public mapLines(lines: number[]): number[] {
    return lines.flatMap((line) => this.findNextPc(line))
  }

  // pc -> line
  constructor(breakpoints: Breakpoint[]) {
    this.maxLine = Math.max(...breakpoints.map(b => b.line))
    this.lineToPc = new Map()
    this.pcToGroup = new Map()

    for (const breakpoint of breakpoints) {
      let lineMap = this.lineToPc.get(breakpoint.line)

      if (!lineMap) {
        lineMap = []

        this.lineToPc.set(breakpoint.line, lineMap)
      }

      const anchorPc = breakpoint.pcs[0]

      if (anchorPc !== undefined) {
        lineMap.push(anchorPc)
      }

      for (const pc of breakpoint.pcs) {
        this.pcToGroup.set(pc, breakpoint)
      }
    }
  }
}

export interface HexRegion {
  name: string
  data: string // base64 encoded
}

interface AssembledRegionsBinary {
  type: 'binary'
  value: string // base64 encoded
}

interface AssembledRegionsSplit {
  type: 'split'
  value: HexRegion[]
}

export type AssembledRegions = AssembledRegionsBinary | AssembledRegionsSplit

export interface HexBinaryResult {
  regions: AssembledRegions | null
  result: AssemblerResult
}

export interface ParameterItemRegular {
  type: 'Register' | 'Immediate' | 'Address'
  value: number
}

export interface ParameterItemOffset {
  type: 'Offset'
  value: { offset: number, register: number }
}

export type ParameterItem = ParameterItemRegular | ParameterItemOffset

export interface InstructionDetails {
  pc: number,
  instruction: number,
  name: string,
  parameters: ParameterItem[]
}

export interface InstructionLineInstruction {
  type: 'Instruction',
  details: InstructionDetails
}

export interface InstructionLineBlank {
  type: 'Blank'
}

export interface InstructionLineComment {
  type: 'Comment',
  message: string
}

export interface InstructionLineLabel {
  type: 'Label',
  name: string
}

export type InstructionLine = InstructionLineInstruction
  | InstructionLineBlank
  | InstructionLineComment
  | InstructionLineLabel

export interface MipsCallbacks {
  consoleWrite(text: string, error: boolean): void
  midiPlay(note: MidiNote): void
}

export interface MipsBackend {
  setCallbacks(callbacks: MipsCallbacks): Promise<void>

  // Insight
  decodeInstruction(pc: number, instruction: number): Promise<InstructionDetails | null>
  disassemblyDetails(bytes: ArrayBuffer): Promise<InstructionLine[]>

  disassembleElf(
    named: string,
    elf: ArrayBuffer
  ): Promise<DisassembleResult>

  assembleText(text: string, path: string | null): Promise<AssemblerResult>
  assembleWithBinary(text: string, path: string | null): Promise<BinaryResult>

  assembleRegions(
    text: string,
    path: string | null,
    options: ExportRegionsOptions
  ): Promise<HexBinaryResult>

  // Execution
  configureDisplay(config: BitmapConfig): Promise<void>
  lastDisplay(): Promise<LastDisplay>

  wakeSync(): Promise<void>

  createExecution(
    text: string,
    path: string | null,
    timeTravel: boolean,
    profile: ExecutionProfile
  ): Promise<MipsExecution>

  close(): void
}

export interface MipsExecution {
  timeTravel: boolean
  profile: ExecutionProfile
  breakpoints: Breakpoints | null

  lastPc(): Promise<number | null>

  configure(): Promise<AssemblerResult | null>
  rewind(count: number): Promise<ExecutionResult | null>
  resume(
    count: number | null,
    breakpoints: number[] | null,
  ): Promise<ExecutionResult | null>
  pause(): Promise<void>
  stop(): Promise<void>
  setBreakpoints(breakpoints: number[]): Promise<void>

  postKey(key: string, up: boolean): Promise<void>
  postInput(text: string): Promise<void>

  memoryAt(
    address: number,
    count: number
  ): Promise<(number | null)[] | null>
  setRegister(register: number, value: number): Promise<void>
  setMemory(address: number, bytes: number[]): Promise<void>

  // Live display, should generally be more performant on tauri.
  readDisplay(width: number, height: number, address: number): Promise<Uint8Array | null>
}
