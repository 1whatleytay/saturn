import { tauri } from '@tauri-apps/api'

import * as MIDI from 'midicube'
import { convertFileSrc } from '@tauri-apps/api/tauri'
;(window as any).MIDI = MIDI

const loadedInstruments = new Map<string, Promise<boolean>>()

export interface MidiNote {
  sync: boolean
  name: string
  instrument: number
  note: number
  duration: number
  volume: number
}

function loadInstrument(instrument: string): void {
  const soundfontUrl = convertFileSrc('', 'midi')

  loadedInstruments.set(
    instrument,
    new Promise((resolve) => {
      MIDI.loadPlugin({
        instrument,
        soundfontUrl,
        targetFormat: 'mp3',
        onerror: () => {
          loadedInstruments.delete(instrument)
          resolve(false)
        },
        onsuccess: () => {
          resolve(true)
        },
      })
    }),
  )
}

export async function playNote(note: MidiNote) {
  const wake = async () => await tauri.invoke('wake_sync')

  if (
    !loadedInstruments.has(note.name) ||
    !(await loadedInstruments.get(note.name))
  ) {
    loadInstrument(note.name)
    return await wake()
  }

  if (note.duration > 0) {
    MIDI.setVolume(0, note.volume)
    MIDI.programChange(0, note.instrument)
    MIDI.noteOn(0, note.note, 127, 0)
    MIDI.noteOff(0, note.note, note.duration)
  }

  if (note.sync) {
    if (note.duration > 0) {
      await new Promise((resolve) =>
        window.setTimeout(resolve, note.duration * 1000),
      )
    }

    await wake()
  }
}
