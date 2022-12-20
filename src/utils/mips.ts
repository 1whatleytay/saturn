import { tauri } from '@tauri-apps/api'

import { v4 as uuid } from 'uuid'

export interface ExecutionProfile {
  elf: ArrayBuffer // not sure what to do here
}

export async function disassembleElf(elf: ArrayBuffer): Promise<string[]> {
  const bytes = Array.from(new Uint8Array(elf))

  return tauri.invoke('disassemble', { bytes })
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
