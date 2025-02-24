import { useSettings } from '../utils/settings'
import { useHighlights } from '../utils/highlights'
import { useStorage } from '../utils/storage'
import { useTabs } from '../utils/tabs'
import { ref } from 'vue'
import { InstructionLine } from '../utils/mips/mips'

export const settings = useSettings()

export const {
  tabsState,
  tab,
  createTab,
  closeTab,
  loadElf,
  saveModal,
  showSettings,
} = useTabs()

export const errorHighlights = useHighlights()

const storageResult = useStorage(errorHighlights, tab)

export const { storage } = storageResult

export const showExportRegionsDialog = ref(false)

export const showFileSaveDialog = ref(false)

export const showFileOpenDialog = ref(false)

export const buildLines = ref(null as InstructionLine[] | null)
