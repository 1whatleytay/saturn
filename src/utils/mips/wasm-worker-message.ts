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
  PauseData

export interface Message {
  id: number
  data: MessageData
}

export enum MessageResponseType {
  Success,
  Failed,
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
