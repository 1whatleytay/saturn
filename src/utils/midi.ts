import { tauri } from '@tauri-apps/api'

import * as MIDI from 'midicube'
import { convertFileSrc } from '@tauri-apps/api/tauri'

(window as any).MIDI = MIDI

const soundfontUrl = convertFileSrc('', 'midi')

const loadedInstruments = new Set<string>()

export interface MidiNote {
  name: string
  instrument: number
  note: number
  duration: number
  volume: number
}

function loadInstrument(instrument: string): Promise<boolean> {
  return new Promise(resolve => {
    MIDI.loadPlugin({
      instrument,
      soundfontUrl,
      onerror: () => resolve(false),
      onsuccess: () => {
        loadedInstruments.add(instrument)

        resolve(true)
      }
    })
  })
}

export async function playNote(note: MidiNote) {
  if (!loadedInstruments.has(note.name)) {
    await loadInstrument(note.name)
  }

  if (note.duration > 0) {
    MIDI.setVolume(0, note.volume)
    MIDI.programChange(0, note.instrument)
    MIDI.noteOn(0, note.note, 127, 0)
    MIDI.noteOff(0, note.note, note.duration)
  }
}
