import {
  AssembledRegions,
  AssemblerResult,
  BinaryResult,
  BitmapConfig, Breakpoint, Breakpoints, DisassembleResult, ExecutionProfile, ExecutionResult,
  HexBinaryResult,
  InstructionDetails,
  InstructionLine,
  LastDisplay, MipsBackend, MipsExecution
} from './mips'
import { ExportRegionsOptions } from '../settings'

import { tauri } from '@tauri-apps/api'

export class TauriExecution implements MipsExecution {
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

        const bytes = new Array(text.length)
        for (let i = 0; i < text.length; i++) {
          bytes[i] = text.charCodeAt(i)
        }

        const result = await tauri.invoke('configure_elf', {
          bytes,
          timeTravel: this.timeTravel
        })

        return result
          ? { status: 'Success', breakpoints: [] }
          : {
            status: 'Error',
            message: 'Configured ELF was not valid',
            body: null,
            marker: null,
          }
      }

      case 'asm': {
        const result = (await tauri.invoke('configure_asm', {
          text: this.text,
          path: this.path,
          timeTravel: this.timeTravel
        })) as AssemblerResult

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

  // Requires Time Travel
  public async lastPc(): Promise<number | null> {
    return await tauri.invoke('last_pc')
  }

  public async rewind(count: number | null): Promise<ExecutionResult | null> {
    return await tauri.invoke('rewind', { count })
  }

  public async resume(
    count: number | null,
    breakpoints: number[] | null,
    listen: (result: AssemblerResult) => void = () => {}
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
      count,
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
      breakpoints: this.breakpoints?.mapLines(breakpoints) ?? [],
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

  public async memoryAt(
    address: number,
    count: number
  ): Promise<(number | null)[] | null> {
    const result = await tauri.invoke('read_bytes', { address, count })

    return result as (number | null)[] | null
  }

  // register: 32 -> hi, 33 -> lo, 34 -> pc
  public async setRegister(register: number, value: number) {
    await tauri.invoke('set_register', { register, value })
  }

  public async setMemory(address: number, bytes: number[]) {
    await tauri.invoke('write_bytes', { address, bytes })
  }

  public constructor(
    public text: string,
    public path: string | null,
    public timeTravel: boolean,
    public profile: ExecutionProfile
  ) {
    switch (profile.kind) {
      case 'elf': {
        const breakpoints = Object.entries(profile.breakpoints).map(
          ([pc, line]) => ({ line, pcs: [parseInt(pc)] } as Breakpoint)
        )

        this.breakpoints = new Breakpoints(breakpoints)

        break
      }

      default:
        this.breakpoints = null
        break
    }
  }
}

export class TauriBackend implements MipsBackend {
  async assembleRegions(text: string, path: string | null, options: ExportRegionsOptions): Promise<HexBinaryResult> {
    const value = (await tauri.invoke('assemble_regions', {
      text, path, options
    })) as [
        AssembledRegions | null,
      AssemblerResult
    ]

    const [regions, result ] = value

    return {
      regions,
      result
    }
  }

  async assembleText(text: string, path: string | null): Promise<AssemblerResult> {
    const result = await tauri.invoke('assemble', { text, path })

    return result as AssemblerResult
  }

  async assembleWithBinary(text: string, path: string | null): Promise<BinaryResult> {
    const result = (await tauri.invoke('assemble_binary', { text, path })) as [
        number[] | null,
      AssemblerResult
    ]

    const [binary, assemblerResult] = result

    return {
      binary: binary ? Uint8Array.from(binary) : null,
      result: assemblerResult,
    }
  }

  async configureDisplay(config: BitmapConfig): Promise<void> {
    await tauri.invoke('configure_display', {
      width: config.width,
      height: config.height,
      address: config.address,
    })
  }

  async decodeInstruction(pc: number, instruction: number): Promise<InstructionDetails | null> {
    return await tauri.invoke('decode_instruction', { pc, instruction }) ?? null
  }

  async disassembleElf(named: string, elf: ArrayBuffer): Promise<DisassembleResult> {
    const bytes = Array.from(new Uint8Array(elf))

    const value = await tauri.invoke('disassemble', { named, bytes })

    return value as DisassembleResult
  }

  async disassemblyDetails(bytes: ArrayBuffer): Promise<InstructionLine[]> {
    return await tauri.invoke('detailed_disassemble', { bytes: Array.from(new Uint8Array(bytes)) })
  }

  async lastDisplay(): Promise<LastDisplay> {
    const result = await tauri.invoke('last_display')

    return result as LastDisplay
  }

  createExecution(
    text: string,
    path: string | null,
    timeTravel: boolean,
    profile: ExecutionProfile
  ): Promise<MipsExecution> {
    return Promise.resolve(
      new TauriExecution(text, path, timeTravel, profile)
    )
  }
}
