import { tauri } from '@tauri-apps/api'

import { Soundfont } from "smplr";
import { convertFileSrc } from '@tauri-apps/api/tauri'

const context = new AudioContext();
const soundfontUrl = convertFileSrc('', 'midi')

const loadedInstruments = new Map<string, Soundfont>()

export interface MidiNote {
  sync: boolean
  name: string
  instrument: number
  note: number
  duration: number
  volume: number
}

async function loadInstrument(instrument: string): Promise<void> {
  const sf = new Soundfont(context, { 
    volumeToGain: (volume: number) => volume,
    instrumentUrl: `${soundfontUrl}/${instrument}-mp3.js`
  });

  loadedInstruments.set(instrument, sf);

  await sf.load;
}

export async function playNote(note: MidiNote) {
  const wake = async () => await tauri.invoke('wake_sync')

  if (!loadedInstruments.has(note.name)) {
    try {
      await loadInstrument(note.name)
    } catch (e) {
      return await wake()
    }
  }

  const sf = loadedInstruments.get(note.name)
  if (note.duration > 0) {
    sf?.start({
      note: note.note,
      duration: note.duration,
      velocity: note.volume / 20 + 2,
    })
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
