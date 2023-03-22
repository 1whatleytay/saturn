import { invoke, tauri } from '@tauri-apps/api'

export interface ElfExecutionProfile {
  kind: 'elf'
  elf: string // not sure what to do here
  breakpoints: Record<number, number>
}

// should only be set after building
export interface AssemblyExecutionProfile {
  kind: 'asm'
}

export type ExecutionProfile = ElfExecutionProfile | AssemblyExecutionProfile;

interface DisassembleResult {
  error: string | null
  lines: string[]
  breakpoints: Record<number, number>
}

export interface LineMarker {
  line: number
  offset: number
}

interface Breakpoint {
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

export type AssemblerResult = AssemblerResultSuccess | AssemblerResultError;

export type BinaryResult = {
  binary: Uint8Array | null,
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
  Finished = 'Finished'
}

export interface ExecutionModeInvalid {
  type: ExecutionModeType.Invalid,
  message: string
}

export interface ExecutionModeFinished {
  type: ExecutionModeType.Finished,
  pc: number
  code: number | null
}

type ExecutionModeOther = ExecutionModeType.Running
  | ExecutionModeType.Breakpoint
  | ExecutionModeType.Paused

export type ExecutionMode = ExecutionModeInvalid
  | ExecutionModeFinished
  | { type: ExecutionModeOther }

export interface Registers {
  pc: number,
  line: number[],
  lo: number,
  hi: number,
}

export interface ExecutionResult {
  mode: ExecutionMode,
  registers: Registers
}

export interface LastDisplay {
  address: number,
  width: number,
  height: number,
  data: number[] | null
}

class Breakpoints {
  public lineToPc: Map<number, number[]>
  public pcToGroup: Map<number, Breakpoint>

  public mapLines(lines: number[]): number[] {
    return lines
      .flatMap(line => this.lineToPc.get(line))
      .filter(point => !!point) as number[]
  }

  // pc -> line
  constructor(breakpoints: Breakpoint[]) {
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

export async function disassembleElf(named: string, elf: ArrayBuffer): Promise<DisassembleResult> {
  const bytes = Array.from(new Uint8Array(elf))

  const value = await tauri.invoke('disassemble', { named, bytes })

  return value as DisassembleResult
}

export async function assembleText(text: string): Promise<AssemblerResult> {
  const result = await tauri.invoke('assemble', { text })

  return result as AssemblerResult
}

export async function assembleWithBinary(text: string): Promise<BinaryResult> {
  const result = await tauri.invoke(
    'assemble_binary', { text }
  ) as [number[] | null, AssemblerResult]

  const [binary, assemblerResult] = result

  return {
    binary: binary ? Uint8Array.from(binary) : null,
    result: assemblerResult
  }
}

export async function configureDisplay(config: BitmapConfig) {
  await invoke('configure_display', {
    width: config.width,
    height: config.height,
    address: config.address
  })
}

export async function lastDisplay(): Promise<LastDisplay> {
  const result = await invoke('last_display')

  return result as LastDisplay
}

export class ExecutionState {
  configured: boolean = false
  public breakpoints: Breakpoints | null

  async configure(): Promise<AssemblerResult | null> {
    if (this.configured) {
      return null
    }

    this.configured = true

    switch (this.profile.kind) {
      case 'elf': {
        const text = window.atob(this.profile.elf)

        const bytes = new Array(text.length);
        for (let i = 0; i < text.length; i++) {
          bytes[i] = text.charCodeAt(i)
        }

        const result = await tauri.invoke('configure_elf', { bytes })

        return result ? { status: 'Success', breakpoints: [] } : {
          status: 'Error',
          message: 'Configured ELF was not valid',
          body: null,
          marker: null
        }
      }

      case 'asm': {
        const result = await tauri.invoke('configure_asm', {
          text: this.text
        }) as AssemblerResult

        if (result.status === 'Success') {
          this.breakpoints = new Breakpoints(result.breakpoints)
        }

        return result
      }

      default:
        break
    }

    throw new Error()
  }

  public async resume(
    count: number | null,
    breakpoints: number[] | null,
    listen: (result: AssemblerResult) => void = () => { }
  ): Promise<ExecutionResult | null> {
    const assemblerResult = await this.configure()

    if (assemblerResult) {
      listen(assemblerResult)

      if (assemblerResult.status === 'Error') {
        return null
      }
    }

    const mappedBreakpoints = breakpoints
      ? this.breakpoints?.mapLines(breakpoints) ?? []
      : []

    const result = await tauri.invoke('resume', {
      breakpoints: mappedBreakpoints,
      count
    })

    return result as ExecutionResult
  }

  // For setting new breakpoints WHILE the machine is running.
  // There's a distinction for some weird technical reason.
  public async setBreakpoints(breakpoints: number[]) {
    if (!this.configured) {
      // Have to invoke resume() with breakpoints anyway.
      return
    }

    await tauri.invoke('swap_breakpoints', {
      breakpoints: this.breakpoints?.mapLines(breakpoints) ?? []
    })
  }

  public async pause() {
    await tauri.invoke('pause')
  }

  public async stop() {
    await tauri.invoke('stop')
  }
  
  public async postKey(key: string, up: boolean) {
    if (key.length !== 1) {
      return
    }
    
    await tauri.invoke('post_key', { key, up })
  }
  
  public async postInput(text: string) {
    if (text.length <= 0) {
      return
    }
    
    await tauri.invoke('post_input', { text })
  }

  public async memoryAt(address: number, count: number): Promise<(number | null)[] | null> {
    const result = await tauri.invoke('read_bytes', { address, count })

    return result as (number | null)[] | null
  }

  // register: 32 -> hi, 33 -> lo, 34 -> pc
  public async setRegister(register: number, value: number) {
    await tauri.invoke('set_register', { register, value })
  }
  
  public constructor(
    public text: string,
    public profile: ExecutionProfile
  ) {
    switch (profile.kind) {
      case 'elf': {
        const breakpoints = Object.entries(profile.breakpoints)
          .map(([pc, line]) => ({ line, pcs: [parseInt(pc)] }) as Breakpoint)

        this.breakpoints = new Breakpoints(breakpoints)

        break
      }

      default:
        this.breakpoints = null
        break
    }
  }
}
