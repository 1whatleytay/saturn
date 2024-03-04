import {
  AssemblerResult,
  BinaryResult,
  BitmapConfig, Breakpoints, DisassembleResult,
  ExecutionProfile, ExecutionResult,
  HexBinaryResult, InstructionDetails, InstructionLine, LastDisplay,
  MipsBackend,
  MipsExecution
} from './mips'
// import WasmWorker from './wasm-worker?worker'
import { ExportRegionsOptions } from '../settings'

interface RequestResponder {
  resolve: (arg: any) => void
  reject: (error: any) => void
}

/**
 * WASM Backend for Saturn (Provides Methods for Assembling/Debugging MIPS Assembly)
 *
 * Why does the WASM Backend use a worker?
 *  - WASM takes control of the active thread when it runs.
 *  - This means when a long request is made (assembling/running a program) the main thread might freeze.
 *  - To avoid the UI becoming unresponsive, requests are made to a worker.
 *  - The worker will then interface with the WASM binary and send a message back to the main thread.
 *
 * You can find the worker source code in wasm-worker.ts (and the wasm binary in wasm/saturn_wasm_bg.wasm)
 */
export class WasmBackend implements MipsBackend {
  worker: Worker
  requestId = 0

  pendingRequests = new Map<number, RequestResponder>()

  async sendRequest<T>(op: string, data: any): Promise<T> {
    const requestId = this.requestId++

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(requestId, {
        resolve: a => resolve(a as T),
        reject: e => reject(e)
      })

      this.worker.postMessage({
        requestId,
        op,
        data
      })
    })
  }

  handleMessage(event: MessageEvent) {
    const { requestId, error, data } = event.data

    const result = this.pendingRequests.get(requestId)

    if (result === undefined) {
      return
    }

    if (error !== undefined && error !== null) {
      result.reject(error)
    } else {
      result.resolve(data)
    }
  }

  async assembleRegions(text: string, path: string | null, options: ExportRegionsOptions): Promise<HexBinaryResult> {
    return  await this.sendRequest<HexBinaryResult>('assemble_regions', { text, path, options })
  }

  async assembleText(text: string, path: string | null): Promise<AssemblerResult> {
    return await this.sendRequest<AssemblerResult>('assemble_text', { text, path })
  }

  async assembleWithBinary(text: string, path: string | null): Promise<BinaryResult> {
    const [binary, assemblerResult] = await this.sendRequest<[
      number[] | null,
      AssemblerResult
    ]>('assemble_binary', { text, path })

    return {
      binary: binary ? Uint8Array.from(binary) : null,
      result: assemblerResult,
    }
  }

  async decodeInstruction(pc: number, instruction: number): Promise<InstructionDetails | null> {
    return await this.sendRequest<InstructionDetails | null>('decode_instruction', { pc, instruction })
  }

  async disassembleElf(named: string, elf: ArrayBuffer): Promise<DisassembleResult> {
    return await this.sendRequest<DisassembleResult>('disassemble', { named, elf: new Uint8Array(elf) })
  }

  async disassemblyDetails(bytes: ArrayBuffer): Promise<InstructionLine[]> {
    return await this.sendRequest<InstructionLine[]>('detailed_disassemble', { bytes: new Uint8Array(bytes) })
  }

  async lastDisplay(): Promise<LastDisplay> {
    return await this.sendRequest<LastDisplay>('last_display', {})
  }

  async configureDisplay(config: BitmapConfig): Promise<void> {
    await this.sendRequest('configure_display', { config })
  }

  async createExecution(text: string, path: string | null, timeTravel: boolean, profile: ExecutionProfile): Promise<MipsExecution> {
    return new WasmExecution(this, text, path, timeTravel, profile)
  }

  constructor() {
    this.worker = null as unknown as Worker // new WasmWorker()

    this.worker.onmessage = event => this.handleMessage(event)
  }
}

export class WasmExecution implements MipsExecution {
  configured = false
  breakpoints: Breakpoints | null = null

  async configure(): Promise<AssemblerResult | null> {
    if (this.configured) {
      return null
    }

    this.configured = true

    switch (this.profile.kind) {
      case 'elf': {
        const text = window.atob(this.profile.elf)

        const bytes = new Uint8Array(text.length)
        for (let i = 0; i < text.length; i++) {
          bytes[i] = text.charCodeAt(i)
        }

        const result = await this.backend.sendRequest<boolean>('configure_elf', {
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
        const result = await this.backend.sendRequest<AssemblerResult>('configure_asm', {
          text: this.text,
          timeTravel: this.timeTravel
        })

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

  lastPc(): Promise<number | null> {
    throw new Error()
  }

  memoryAt(address: number, count: number): Promise<(number | null)[] | null> {
    throw new Error()
  }

  setBreakpoints(breakpoints: number[]): Promise<void> {
    throw new Error()
  }

  setMemory(address: number, bytes: number[]): Promise<void> {
    throw new Error()
  }

  setRegister(register: number, value: number): Promise<void> {
    throw new Error()
  }

  postInput(text: string): Promise<void> {
    throw new Error()
  }

  postKey(key: string, up: boolean): Promise<void> {
    throw new Error()
  }

  pause(): Promise<void> {
    throw new Error()
  }

  stop(): Promise<void> {
    throw new Error()
  }

  resume(count: number | null, breakpoints: number[] | null, listen: (result: AssemblerResult) => void): Promise<ExecutionResult | null> {
    throw new Error()
  }

  rewind(count: number | null): Promise<ExecutionResult | null> {
    throw new Error()
  }

  constructor(
    public backend: WasmBackend,
    public text: string,
    public path: string | null,
    public timeTravel: boolean,
    public profile: ExecutionProfile
  ) { }
}
