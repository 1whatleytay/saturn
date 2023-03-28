import { tauri } from '@tauri-apps/api'

import * as MIDI from 'midicube'
import { convertFileSrc } from '@tauri-apps/api/tauri'

;(window as any).MIDI = MIDI

const soundfontUrl = convertFileSrc('', 'midi')

const loadedInstruments = new Set<string>()

export interface MidiNote {
  sync: boolean
  name: string
  instrument: number
  note: number
  duration: number
  volume: number
}

function loadInstrument(instrument: string): Promise<boolean> {
  return new Promise((resolve) => {
    MIDI.loadPlugin({
      instrument,
      soundfontUrl,
      targetFormat: 'mp3',
      onerror: () => resolve(false),
      onsuccess: () => {
        loadedInstruments.add(instrument)

        resolve(true)
      },
    })
  })
}

export async function playNote(note: MidiNote) {
  const wake = async () => await tauri.invoke('wake_sync')

  if (!loadedInstruments.has(note.name)) {
    if (!(await loadInstrument(note.name))) {
      console.error(`Failed to load instrument ${note.name}`)

      return await wake()
    }
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
        window.setTimeout(resolve, note.duration * 1000)
      )
    }

    await wake()
  }
}
