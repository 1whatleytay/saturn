import { postBuildMessage } from '../debug'
import {
  assemblyFilter,
  selectOpenElf,
} from '../query/access-manager/access-manager-tauri'
import { consoleData } from '../../state/console-data'
import { backend } from '../../state/backend'
import {
  closeTab,
  createTab,
  loadElf,
  showExportRegionsDialog,
  showSettings,
  tab,
  tabsState,
} from '../../state/state'
import {
  AccessFile,
  accessWriteText,
  selectOpenFile,
  selectSaveDestination,
} from '../query/access-manager'
import { EditorTab } from '../tabs'

export enum PromptType {
  NeverPrompt,
  PromptWhenNeeded,
  ForcePrompt,
}

export interface Accelerator {
  command: boolean
  shift: boolean
  key: string
}

// 'new-tab'
export function newTab() {
  createTab('Untitled', '')
}

// 'close-tab'
export function closeCurrentTab() {
  if (tabsState.selected) {
    closeTab(tabsState.selected)
  }
}

// 'build' - build
// 'run' - resume
// 'step' - step
// 'pause' - pause
// 'stop' - stop

// 'assemble'
export async function assemble() {
  const current = tab()

  const result = await backend.assembleWithBinary(
    current?.state?.doc?.toString() ?? '',
    current?.path ?? null,
  )

  if (result.binary) {
    const name = tab()?.title
    const extended = name ? `${name}.elf` : 'Untitled Elf'

    await loadElf(extended, result.binary.buffer)
  }

  consoleData.showConsole = true
  postBuildMessage(result.result)
}

// 'disassemble'
export async function disassemble() {
  const result = await selectOpenElf()

  if (!result) {
    return
  }

  const { name, data } = result

  await loadElf(name ?? 'Untitled', data.buffer)
}

// 'toggle-console'
export function toggleConsole() {
  consoleData.showConsole = !consoleData.showConsole
}

// 'toggle-settings'
export function toggleSettings() {
  showSettings.value = !showSettings.value
}

// 'export-hex'
export async function exportHex() {
  showExportRegionsDialog.value = !showExportRegionsDialog.value
}

// 'open-file' - tauri required
export async function openFile() {
  const result = await selectOpenFile('')

  if (!result) {
    return
  }

  await openTab(result)
}

export async function openTab(file: AccessFile<string | Uint8Array>) {
  const { name: inName, path, data } = file
  const name = inName ?? 'Untitled'

  const existing = tabsState.tabs.find((tab) => tab.path === path)

  if (existing) {
    tabsState.selected = existing.uuid
    return
  }

  switch (typeof data) {
    case 'string':
      createTab(name, data, path)
      break

    default:
      await loadElf(name, data.buffer)
      break
  }
}

export async function saveTab(
  current: EditorTab,
  type: PromptType = PromptType.PromptWhenNeeded,
): Promise<boolean> {
  if (type === PromptType.NeverPrompt && !current.path) {
    return true
  }

  if (type === PromptType.ForcePrompt || !current.path) {
    const result = await selectSaveDestination('Save File', assemblyFilter)

    if (!result) {
      return false
    }

    const { name, path } = result

    current.title = name ?? 'Untitled'
    current.path = path
  }

  if (current.path.startsWith('remote://')) {
    return false
  }

  const data = current.doc.toString()

  await accessWriteText(current.path, data)

  current.marked = false // Remove "needs saving" marker

  return true
}

export async function saveCurrentTab(
  prompt: PromptType = PromptType.PromptWhenNeeded,
) {
  const current = tab()

  if (current) {
    await saveTab(current, prompt)
  }
}

// 'save'
export async function save() {
  // duplicating this function for consistency
  await saveCurrentTab()
}

// 'save-as'
export async function saveAs() {
  await saveCurrentTab(PromptType.ForcePrompt)
}
