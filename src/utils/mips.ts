import { tauri } from '@tauri-apps/api'

export interface ElfExecutionProfile {
  kind: 'elf',
  elf: ArrayBuffer // not sure what to do here
  breakpoints: Record<number, number>
}

// should only be set after building
export interface AssemblyExecutionProfile {
  kind: 'assembly',
  breakpoints: Record<number, number>
}

export type ExecutionProfile = ElfExecutionProfile; // | AssemblyExecutionProfile

interface DisassembleResult {
  error: string | null,
  lines: string[],
  breakpoints: Record<number, number>
}

export async function disassembleElf(named: string, elf: ArrayBuffer): Promise<DisassembleResult> {
  const bytes = Array.from(new Uint8Array(elf))

  const value = await tauri.invoke('disassemble', { named, bytes })

  return value as DisassembleResult
}

export enum ExecutionMode {
  Running = 'Running',
  Invalid = 'Invalid',
  Paused = 'Paused',
  Breakpoint = 'Breakpoint'
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
  started: boolean = false

  async configure() {
    if (this.started) {
      return
    }

    this.started = true

    const result = await tauri.invoke('configure_elf', {
      bytes: Array.from(new Uint8Array(this.profile.elf))
    })

    if (!result) {
      console.error('Failed to configure interpreter.')
    }
  }

  public async resume(breakpoints: number[]): Promise<ExecutionResult> {
    await this.configure()

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
    private profile: ExecutionProfile
  ) { }
}
