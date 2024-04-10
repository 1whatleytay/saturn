import { useSettings } from '../utils/settings'
import { useHighlights } from '../utils/highlights'
import { regular } from '../utils/query/text-size'
import { useStorage } from '../utils/storage'

import { useTabs } from '../utils/tabs'
import { GotoMessage, useGoto } from '../utils/goto'
import { ref } from 'vue'

export const settings = useSettings()

function widthQuery(text: string) {
  return regular.calculate(text).width
}

export const {
  tabsState,
  tab,
  createTab,
  closeTab,
  loadElf,
  saveModal,
  showSettings
} = useTabs()

export const errorHighlights = useHighlights(widthQuery)
export const gotoHighlights = useHighlights<GotoMessage>(widthQuery)


const storageResult = useStorage(errorHighlights, tab, onDirty)

export const { editor, storage, suggestionsStorage } = storageResult

function onDirty(line: number, deleted: number, insert: string[]) {

}
// watch(() => {
//   // const cursor = cursorIndex()
//   // const line = tab()?.lines[cursor.line]

//   // const index = line ? Math.min(line.length, cursor.index) : cursor.index

//   // return { line: cursor.line, index }
// }, updateCursorSymbol)

export const goto = useGoto(gotoHighlights, storageResult)

export const showExportRegionsDialog = ref(false)
