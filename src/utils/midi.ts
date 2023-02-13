import { tauri } from '@tauri-apps/api'

import * as MIDI from 'midicube'
import { convertFileSrc } from '@tauri-apps/api/tauri'

(window as any).MIDI = MIDI

export async function install() {
  const result = await tauri.invoke('midi_install', {
    instruments: ['trombone', 'synth_choir', 'pad_4_choir', 'koto']
  })

  console.log(result)
}

const soundfontUrl = convertFileSrc('midi/FatBoy/', 'midi')

export async function loadInstrument() {
  MIDI.loadPlugin({
    instrument: 'acoustic_grand_piano',
    soundfontUrl,
    onerror: console.error,
    onsuccess: () => {
      console.log('done loading!')

      var delay = 0; // play one note every quarter second
      var note = 50; // the MIDI note
      var velocity = 127; // how hard the note hits
      // play the note
      MIDI.setVolume(0, 127);
      MIDI.noteOn(0, note, velocity, delay);
      MIDI.noteOff(0, note, delay + 0.20);
    }
  })
}

export async function playNote() {

}
