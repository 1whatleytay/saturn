import { reactive, watch } from 'vue'
import { Editor } from '../utils/editor'
import { tab } from './tabs-state'
import { cursor } from './cursor-state'

export const storage = reactive({
  editor: createEditor(),
})

function createEditor(): Editor {
  return new Editor(
    tab()?.lines ?? ['Nothing yet.'],
    () => cursor
  )
}

watch(() => tab(), () => {
  storage.editor = createEditor()
})
