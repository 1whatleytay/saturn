import { tauri } from '@tauri-apps/api'

import { v4 as uuid } from 'uuid'

export interface ExecutionProfile {
  elf: ArrayBuffer // not sure what to do here
  breakpoints: Record<number, number>
}

interface DisassembleResult {
  error: string | null,
  lines: string[],
  breakpoints: Record<number, number>
}

export async function disassembleElf(named: string, elf: ArrayBuffer): Promise<DisassembleResult> {
  const bytes = Array.from(new Uint8Array(elf))

  const value = await tauri.invoke('disassemble', { named, bytes })
  console.log(value)

  return value as DisassembleResult
}

enum ExecutionMode {
  Running = 'Running',
  Invalid = 'Invalid',
  Paused = 'Paused',
  Breakpoint = 'Breakpoint',
}

export interface ExecutionResult {
  mode: ExecutionMode,

  pc: number,
  registers: number[],
  lo: number,
  hi: number
}

export class ExecutionState {
  started: boolean = false

  async configure() {
    if (this.started) {
      return
    }

    this.started = true

    const result = await tauri.invoke('configure', {
      bytes: Array.from(new Uint8Array(this.profile.elf))
    })

    if (!result) {
      console.error('Failed to configure interpreter.')
    }
  }

  public async run(breakpoints: number[]): Promise<ExecutionResult> {
    await this.configure()

    const result = await tauri.invoke('resume', {
      breakpoints: breakpoints
        .map(point => this.profile.breakpoints[point])
        .filter(point => !!point)
    })

    return result as ExecutionResult
  }

  public async close() {
    await tauri.invoke('pause')
  }

  public constructor(
    private profile: ExecutionProfile
  ) { }
}
