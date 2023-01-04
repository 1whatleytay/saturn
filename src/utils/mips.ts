import { tauri } from '@tauri-apps/api'

export interface ElfExecutionProfile {
  kind: 'elf'
  elf: ArrayBuffer // not sure what to do here
  breakpoints: Record<number, number>
}

// should only be set after building
export interface AssemblyExecutionProfile {
  kind: 'asm'
  breakpoints: Record<number, number>
}

export type ExecutionProfile = ElfExecutionProfile | AssemblyExecutionProfile;

interface DisassembleResult {
  error: string | null
  lines: string[]
  breakpoints: Record<number, number>
}

interface LineMarker {
  line: number
  offset: number
}

interface AssemblerResultSuccess {
  status: 'Success'
  breakpoints: Record<number, number>
}

interface AssemblerResultError {
  status: 'Error'
  marker: LineMarker | null
  body: string | null
  message: string
}

type AssemblerResult = AssemblerResultSuccess | AssemblerResultError;

export async function disassembleElf(named: string, elf: ArrayBuffer): Promise<DisassembleResult> {
  const bytes = Array.from(new Uint8Array(elf))

  const value = await tauri.invoke('disassemble', { named, bytes })

  return value as DisassembleResult
}

export enum ExecutionMode {
  Running = 'Running',
  Invalid = 'Invalid',
  Paused = 'Paused',
  Breakpoint = 'Breakpoint',
  BuildFailed = 'BuildFailed'
}

export interface ExecutionResult {
  mode: ExecutionMode,

  pc: number,
  registers: number[],
  lo: number,
  hi: number,
}

export function defaultResult(mode: ExecutionMode): ExecutionResult {
  return {
    mode,
    pc: 0,
    registers: Array(32).fill(0),
    lo: 0,
    hi: 0
  }
}

export class ExecutionState {
  configured: boolean = false
  breakpoints: Record<number, number> | null = null

  async configure(): Promise<boolean> {
    if (this.configured) {
      return true
    }

    this.configured = true

    switch (this.profile.kind) {
      case 'elf': {
        const result = await tauri.invoke('configure_elf', {
          bytes: Array.from(new Uint8Array(this.profile.elf))
        })

        if (!result) {
          console.error('Failed to configure interpreter.')
        }

        return !!result
      }

      case 'asm': {
        const result = await tauri.invoke('configure_asm', {
          text: this.text
        }) as AssemblerResult

        switch (result.status) {
          case 'Success':
            this.breakpoints = result.breakpoints

            return true

          case 'Error':
            return false
        }

        return false
      }

      default: throw new Error()
    }
  }

  public async resume(breakpoints: number[]): Promise<ExecutionResult> {
    if (!await this.configure()) {
      return defaultResult(ExecutionMode.BuildFailed)
    }

    const result = await tauri.invoke('resume', {
      breakpoints: breakpoints
        .map(point => this.profile.breakpoints[point])
        .filter(point => !!point)
    })

    return result as ExecutionResult
  }

  public async pause(): Promise<ExecutionResult> {
    const result = await tauri.invoke('pause')

    return result as ExecutionResult
  }

  public async step(): Promise<ExecutionResult> {
    const result = await tauri.invoke('step')

    return result as ExecutionResult
  }

  public async stop() {
    await tauri.invoke('stop')
  }

  public async memoryAt(address: number, count: number): Promise<(number | null)[] | null> {
    const result = await tauri.invoke('read_bytes', { address, count })

    return result as (number | null)[] | null
  }

  public constructor(
    private text: string,
    private profile: ExecutionProfile
  ) { }
}
