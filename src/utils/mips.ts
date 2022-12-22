import { tauri } from '@tauri-apps/api'

import { v4 as uuid } from 'uuid'

export interface ExecutionProfile {
  elf: ArrayBuffer // not sure what to do here
  breakpoints: Map<number, number>
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

export enum ExecutionResult {
  Finished,
  Error,
  Breakpoint
}

export class ExecutionState {
  private uuid: string

  public async run(breakpoints: number[]): Promise<ExecutionResult> {
    // !

    return ExecutionResult.Breakpoint
  }

  public async close() {
    // !
  }

  public constructor(
    private profile: ExecutionProfile
  ) {
    this.uuid = uuid()
  }
}
