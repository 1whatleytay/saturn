declare module 'midicube' {
  interface PluginConfig {
    instrument?: string
    instruments?: string[]
    soundfontUrl?: string
    targetFormat?: string
    onerror?: (error: string) => void
    onsuccess?: () => void
    onprogress?: (state: any, progress: any) => void
  }

  function loadPlugin(config: PluginConfig)

  function setVolume(low: number, high: number)
  function programChange(channel: number, instrument: number)
  function noteOn(channel: number, note: number, velocity: number, delay?: number)
  function noteOff(channel: number, note: number, delay?: number)
}
