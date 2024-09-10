import * as backend from './wasm/saturn_wasm'
import {
  AssembledRegions,
  AssemblerResult,
  BinaryResult,
  DisassembleResult,
  ExecutionModeType,
  ExecutionResult,
  HexBinaryResult,
  InstructionDetails,
  InstructionLine,
  LastDisplay
} from './mips'
import {
  AssembleBinaryData,
  AssembleRegionsData,
  AssembleTextData,
  ConfigureAsmData,
  ConfigureDisplayData,
  ConfigureElfData,
  DecodeInstructionData,
  DetailedDisassembleData,
  DisassembleData,
  Message,
  MessageData,
  MessageOp,
  MessageResponse,
  PostInputData,
  PostKeyData,
  ReadBytesData, ReadDisplayData,
  ResumeData, RewindData,
  SetBreakpointsData,
  SetRegisterData,
  WriteBytesData
} from './wasm-worker-message'

backend.initialize()

// Runner/Execution State (Automatically Freed with the Worker Memory)
const runner = new backend.Runner();

function assembleRegions({ text, options }: AssembleRegionsData): HexBinaryResult {
  const [regions, result] = backend.assemble_regions(text, options) as [
    AssembledRegions | null,
    AssemblerResult
  ]

  return {
    regions,
    result
  }
}

function assembleText({ text }: AssembleTextData): AssemblerResult {
  return backend.assemble_text(text) as AssemblerResult
}

function assembleBinary({ text }: AssembleBinaryData): BinaryResult {
  return backend.assemble_binary(text) as BinaryResult
}

function decodeInstruction({ pc, instruction }: DecodeInstructionData): InstructionDetails | null {
  return backend.decode_instruction(pc, instruction) as InstructionDetails | null
}

function disassemble({ named, bytes }: DisassembleData): DisassembleResult {
  return backend.disassemble(named ?? undefined, bytes) as DisassembleResult
}

function detailedDisassemble({ bytes }: DetailedDisassembleData): InstructionLine[] {
  return backend.detailed_disassemble(bytes) as InstructionLine[]
}

function configureDisplay({ config }: ConfigureDisplayData) {
  runner.configure_display(config.address, config.width, config.height)
}

function lastDisplay(): LastDisplay {
  return runner.last_display()
}

function configureElf({ bytes, timeTravel }: ConfigureElfData): boolean {
  return runner.configure_elf(bytes, timeTravel)
}

function configureAsm({ text, timeTravel }: ConfigureAsmData): AssemblerResult {
  return runner.configure_asm(text, timeTravel)
}

async function resume({ count, breakpoints }: ResumeData): Promise<ExecutionResult | null> {
  const batchSize = 1200 // worth adjusting this batch size

  let instructionsExecuted = 0

  const breaks = breakpoints === null ? undefined : new Uint32Array(breakpoints)

  let result: ExecutionResult | null = null

  let firstRun = true

  while (count === null || instructionsExecuted < count) {
    const instructionsToExecute = count === null ? batchSize : Math.min(count - instructionsExecuted, batchSize)

    result = await runner.resume(instructionsToExecute, firstRun ? breaks : undefined, firstRun, count !== null) as ExecutionResult | null

    firstRun = false

    if (result === null) {
      return null
    }

    if (result.mode.type !== ExecutionModeType.Running) {
      return result
    }

    instructionsExecuted += batchSize

    await new Promise<void>(resolve => setTimeout(resolve, 0))
  }

  return result
}

function stop() {
  runner.stop()
}

function pause() {
  runner.pause()
}

function lastPc(): number | null {
  return runner.last_pc() ?? null
}

function readBytes({ address, count }: ReadBytesData): (number | null)[] | null {
  return runner.read_bytes(address, count)
}

function writeBytes({ address, bytes }: WriteBytesData) {
  runner.write_bytes(address, bytes)
}

function setRegister({ register, value }: SetRegisterData) {
  runner.set_register(register, value)
}

function setBreakpoints({ breakpoints }: SetBreakpointsData) {
  console.log({ breakpoints })

  runner.set_breakpoints(breakpoints)
}

function postInput({ text }: PostInputData) {
  runner.post_input(text)
}

function postKey({ key, up }: PostKeyData) {
  runner.post_key(key, up)
}

function wakeSync() {
  runner.wake_sync()
}

function rewind({ count }: RewindData): ExecutionResult | null {
  return runner.rewind(count)
}

function readDisplay({ width, height, address }: ReadDisplayData) {
  return runner.read_display(address, width, height)
}

async function dispatchOp(data: MessageData): Promise<any> {
  switch (data.op) {
    case MessageOp.AssembleRegions: return assembleRegions(data)
    case MessageOp.AssembleText: return assembleText(data)
    case MessageOp.AssembleBinary: return assembleBinary(data)
    case MessageOp.DecodeInstruction: return decodeInstruction(data)
    case MessageOp.Disassemble: return disassemble(data)
    case MessageOp.DetailedDisassemble: return detailedDisassemble(data)
    case MessageOp.ConfigureDisplay: return configureDisplay(data)
    case MessageOp.LastDisplay: return lastDisplay()
    case MessageOp.ConfigureElf: return configureElf(data)
    case MessageOp.ConfigureAsm: return configureAsm(data)
    case MessageOp.Resume: return await resume(data)
    case MessageOp.Stop: return stop()
    case MessageOp.Pause: return pause()
    case MessageOp.LastPc: return lastPc()
    case MessageOp.ReadBytes: return readBytes(data)
    case MessageOp.WriteBytes: return writeBytes(data)
    case MessageOp.SetRegister: return setRegister(data)
    case MessageOp.SetBreakpoints: return setBreakpoints(data)
    case MessageOp.PostInput: return postInput(data)
    case MessageOp.PostKey: return postKey(data)
    case MessageOp.WakeSync: return wakeSync()
    case MessageOp.Rewind: return rewind(data)
    case MessageOp.ReadDisplay: return readDisplay(data)
  }
}

async function handleMessage(event: MessageEvent) {
  const { id, data } = event.data as Message

  try {
    const value = await dispatchOp(data)

    postMessage({
      id,
      success: true,
      data: value
    } satisfies MessageResponse)
  } catch (error) {
    postMessage({
      id,
      success: false,
      error
    } satisfies MessageResponse)
  }
}

onmessage = handleMessage
