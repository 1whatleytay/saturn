import {
  AssemblerResult,
  BinaryResult,
  BitmapConfig,
  Breakpoints,
  DisassembleResult,
  ExecutionProfile,
  ExecutionResult,
  HexBinaryResult,
  InstructionDetails,
  InstructionLine,
  LastDisplay,
  MipsBackend, MipsCallbacks,
  MipsExecution
} from './mips'
import WasmWorker from './wasm-worker?worker'
import { ExportRegionsOptions } from '../settings'
import {
  Message,
  MessageData,
  MessageEventData, MessageEventOp,
  MessageOp,
  MessageResponse,
  MessageResponseKind
} from './wasm-worker-message'

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

  callbacks: MipsCallbacks | null = null

  pendingRequests = new Map<number, RequestResponder>()

  async setCallbacks(callbacks: MipsCallbacks): Promise<void> {
    this.callbacks = callbacks
  }

  async sendRequest<T>(data: MessageData): Promise<T> {
    const requestId = this.requestId++

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(requestId, {
        resolve: (a) => resolve(a as T),
        reject: (e) => reject(e),
      })

      this.worker.postMessage({
        id: requestId,
        data,
      } satisfies Message)
    })
  }

  handleEvent(data: MessageEventData) {
    if (!this.callbacks) {
      return
    }

    switch (data.op) {
      case MessageEventOp.ConsoleWrite:
        this.callbacks.consoleWrite(data.text, data.error)
        break
      case MessageEventOp.MidiPlay:
        this.callbacks.midiPlay(data.note)
        break
    }
  }

  handleMessage(event: MessageEvent) {
    const response = event.data as MessageResponse

    if (response.kind === MessageResponseKind.Event) {
      this.handleEvent(response.data)

      return
    }

    const result = this.pendingRequests.get(response.id)

    if (result === undefined) {
      return
    }

    this.pendingRequests.delete(response.id)

    switch (response.kind) {
      case MessageResponseKind.Success:
        result.resolve(response.data)
        break

      case MessageResponseKind.Failure:
        result.reject(response.error)
        break
    }
  }

  async assembleRegions(
    text: string,
    path: string | null,
    options: ExportRegionsOptions
  ): Promise<HexBinaryResult> {
    return await this.sendRequest<HexBinaryResult>({
      op: MessageOp.AssembleRegions,
      text,
      path,
      options,
    })
  }

  async assembleText(
    text: string,
    path: string | null
  ): Promise<AssemblerResult> {
    return await this.sendRequest<AssemblerResult>({
      op: MessageOp.AssembleText,
      text,
      path,
    })
  }

  async assembleWithBinary(
    text: string,
    path: string | null
  ): Promise<BinaryResult> {
    const [binary, assemblerResult] = await this.sendRequest<
      [number[] | null, AssemblerResult]
    >({ op: MessageOp.AssembleBinary, text, path })

    return {
      binary: binary ? Uint8Array.from(binary) : null,
      result: assemblerResult,
    }
  }

  async decodeInstruction(
    pc: number,
    instruction: number
  ): Promise<InstructionDetails | null> {
    return await this.sendRequest<InstructionDetails | null>({
      op: MessageOp.DecodeInstruction,
      pc,
      instruction,
    })
  }

  async disassembleElf(
    named: string,
    elf: ArrayBuffer
  ): Promise<DisassembleResult> {
    return await this.sendRequest<DisassembleResult>({
      op: MessageOp.Disassemble,
      named,
      bytes: new Uint8Array(elf),
    })
  }

  async disassemblyDetails(bytes: ArrayBuffer): Promise<InstructionLine[]> {
    return await this.sendRequest<InstructionLine[]>({
      op: MessageOp.DetailedDisassemble,
      bytes: new Uint8Array(bytes),
    })
  }

  async lastDisplay(): Promise<LastDisplay> {
    return await this.sendRequest<LastDisplay>({ op: MessageOp.LastDisplay })
  }

  async configureDisplay(config: BitmapConfig): Promise<void> {
    await this.sendRequest({ op: MessageOp.ConfigureDisplay, config })
  }

  wakeSync(): Promise<void> {
    return this.sendRequest({
      op: MessageOp.WakeSync,
    })
  }

  async createExecution(
    text: string,
    path: string | null,
    timeTravel: boolean,
    profile: ExecutionProfile
  ): Promise<MipsExecution> {
    return new WasmExecution(this, text, path, timeTravel, profile)
  }

  constructor() {
    this.worker = new WasmWorker()

    this.worker.onmessage = (event) => this.handleMessage(event)
  }

  close() {
    this.worker.terminate()
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

        const result = await this.backend.sendRequest<boolean>({
          op: MessageOp.ConfigureElf,
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
        const result = await this.backend.sendRequest<AssemblerResult>({
          op: MessageOp.ConfigureAsm,
          text: this.text,
          timeTravel: this.timeTravel
        })

        if (result.status === 'Success') {
          this.breakpoints = new Breakpoints(result.breakpoints)
        }

        return result
      }

      default:
        throw new Error()
    }
  }

  lastPc(): Promise<number | null> {
    return this.backend.sendRequest<number | null>({ op: MessageOp.LastPc })
  }

  memoryAt(address: number, count: number): Promise<(number | null)[] | null> {
    return this.backend.sendRequest<(number | null)[] | null>({
      op: MessageOp.ReadBytes,
      address,
      count
    })
  }

  setBreakpoints(breakpoints: number[]): Promise<void> {
    const mappedBreakpoints = this.breakpoints?.mapLines(breakpoints) ?? []

    return this.backend.sendRequest({
      op: MessageOp.SetBreakpoints,
      breakpoints: new Uint32Array(mappedBreakpoints)
    })
  }

  setMemory(address: number, bytes: number[]): Promise<void> {
    return this.backend.sendRequest({
      op: MessageOp.WriteBytes,
      address,
      bytes: new Uint8Array(bytes)
    })
  }

  setRegister(register: number, value: number): Promise<void> {
    return this.backend.sendRequest({
      op: MessageOp.SetRegister,
      register,
      value
    })
  }

  postInput(text: string): Promise<void> {
    return this.backend.sendRequest({
      op: MessageOp.PostInput,
      text
    })
  }

  postKey(key: string, up: boolean): Promise<void> {
    return this.backend.sendRequest({
      op: MessageOp.PostKey,
      key,
      up
    })
  }

  pause(): Promise<void> {
    return this.backend.sendRequest({ op: MessageOp.Pause })
  }

  stop(): Promise<void> {
    return this.backend.sendRequest({ op: MessageOp.Stop })
  }

  resume(count: number | null, breakpoints: number[] | null): Promise<ExecutionResult | null> {
    const mappedBreakpoints = breakpoints
      ? this.breakpoints?.mapLines(breakpoints) ?? []
      : []


    return this.backend.sendRequest<ExecutionResult | null>({ op: MessageOp.Resume, count, breakpoints: mappedBreakpoints })
  }

  rewind(count: number): Promise<ExecutionResult | null> {
    return this.backend.sendRequest<ExecutionResult | null>({
      op: MessageOp.Rewind,
      count
    })
  }

  readDisplay(width: number, height: number, address: number): Promise<Uint8Array | null> {
    return this.backend.sendRequest<Uint8Array | null>({
      op: MessageOp.ReadDisplay,
      width,
      height,
      address
    })
  }

  constructor(
    public backend: WasmBackend,
    public text: string,
    public path: string | null,
    public timeTravel: boolean,
    public profile: ExecutionProfile
  ) { }
}
