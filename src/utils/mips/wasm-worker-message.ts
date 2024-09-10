import { ExportRegionsOptions } from '../settings'
import { BitmapConfig } from './mips'

export enum MessageOp {
  AssembleRegions,
  AssembleText,
  AssembleBinary,
  DecodeInstruction,
  Disassemble,
  DetailedDisassemble,
  ConfigureDisplay,
  LastDisplay,
  ConfigureElf,
  ConfigureAsm,
  Resume,
  Stop,
  Pause,
  LastPc,
  ReadBytes,
  WriteBytes,
  SetRegister,
  SetBreakpoints,
  PostInput,
  PostKey,
  WakeSync,
  Rewind,
  ReadDisplay,
}

export interface AssembleRegionsData {
  op: MessageOp.AssembleRegions

  text: string
  path: string | null
  options: ExportRegionsOptions
}

export interface AssembleTextData {
  op: MessageOp.AssembleText

  text: string
  path: string | null
}

export interface AssembleBinaryData {
  op: MessageOp.AssembleBinary

  text: string
  path: string | null
}

export interface DecodeInstructionData {
  op: MessageOp.DecodeInstruction

  pc: number
  instruction: number
}

export interface DisassembleData {
  op: MessageOp.Disassemble

  named: string | null
  bytes: Uint8Array
}

export interface DetailedDisassembleData {
  op: MessageOp.DetailedDisassemble

  bytes: Uint8Array
}

export interface ConfigureDisplayData {
  op: MessageOp.ConfigureDisplay

  config: BitmapConfig
}

export interface LastDisplayData {
  op: MessageOp.LastDisplay
}

export interface ConfigureElfData {
  op: MessageOp.ConfigureElf

  bytes: Uint8Array
  timeTravel: boolean
}

export interface ConfigureAsmData {
  op: MessageOp.ConfigureAsm

  text: string
  timeTravel: boolean
}

export interface ResumeData {
  op: MessageOp.Resume

  count: number | null
  breakpoints: number[] | null
}

export interface StopData {
  op: MessageOp.Stop
}

export interface PauseData {
  op: MessageOp.Pause
}

export interface LastPcData {
  op: MessageOp.LastPc
}

export interface ReadBytesData {
  op: MessageOp.ReadBytes
  address: number
  count: number
}

export interface WriteBytesData {
  op: MessageOp.WriteBytes
  address: number
  bytes: Uint8Array
}

export interface SetRegisterData {
  op: MessageOp.SetRegister
  register: number
  value: number
}

export interface SetBreakpointsData {
  op: MessageOp.SetBreakpoints
  breakpoints: Uint32Array
}

export interface PostInputData {
  op: MessageOp.PostInput
  text: string
}

export interface PostKeyData {
  op: MessageOp.PostKey
  key: string
  up: boolean
}

export interface WakeSyncData {
  op: MessageOp.WakeSync
}

export interface RewindData {
  op: MessageOp.Rewind
  count: number
}

export interface ReadDisplayData {
  op: MessageOp.ReadDisplay
  width: number
  height: number
  address: number
}

export type MessageData =
  AssembleRegionsData |
  AssembleTextData |
  AssembleBinaryData |
  DecodeInstructionData |
  DisassembleData |
  DetailedDisassembleData |
  ConfigureDisplayData |
  LastDisplayData |
  ConfigureElfData |
  ConfigureAsmData |
  ResumeData |
  StopData |
  PauseData |
  LastPcData |
  ReadBytesData |
  WriteBytesData |
  SetRegisterData |
  SetBreakpointsData |
  PostInputData |
  PostKeyData |
  WakeSyncData |
  RewindData |
  ReadDisplayData

export interface Message {
  id: number
  data: MessageData
}

export interface MessageResponseSuccess {
  id: number
  success: true
  data: unknown
}

export interface MessageResponseFailure {
  id: number
  success: false
  error: unknown
}

export type MessageResponse = MessageResponseSuccess | MessageResponseFailure
