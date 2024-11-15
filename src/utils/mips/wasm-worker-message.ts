import { type ExportRegionsOptions } from '../settings'
import { type BitmapConfig } from './mips'
import { type MidiNote } from '../midi'

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
  PlatformShortcuts,
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
  register: number | null
}

export interface PlatformShortcutsData {
  op: MessageOp.PlatformShortcuts
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
  ReadDisplayData |
  PlatformShortcutsData

export enum MessageEventOp {
  ConsoleWrite,
  MidiPlay,
  Ready,
}

export interface MessageEventReady {
  op: MessageEventOp.Ready
}

export interface MessageEventConsoleWrite {
  op: MessageEventOp.ConsoleWrite
  text: string
  error: boolean
}

export interface MessageEventMidiPlay {
  op: MessageEventOp.MidiPlay
  note: MidiNote
}

export type MessageEventData =
  MessageEventConsoleWrite |
  MessageEventMidiPlay |
  MessageEventReady

export interface Message {
  id: number
  data: MessageData
}

export enum MessageResponseKind {
  Success,
  Failure,
  Event,
}

export interface MessageResponseSuccess {
  id: number
  kind: MessageResponseKind.Success
  data: unknown
}

export interface MessageResponseEvent {
  kind: MessageResponseKind.Event
  data: MessageEventData
}

export interface MessageResponseFailure {
  id: number
  kind: MessageResponseKind.Failure
  error: unknown
}

export type MessageResponse = MessageResponseSuccess
  | MessageResponseFailure
  | MessageResponseEvent
