<template>
  <DialogRoot v-model:open="showFileOpenDialog">
    <DialogPortal>
      <DialogOverlay
        class="fixed inset-0 z-[350] h-screen w-screen bg-black bg-opacity-40 opacity-0 transition-opacity duration-500 data-[state=open]:opacity-100"
      />
      <DialogContent
        class="data-[state=open]:animate-contentShow fixed left-[50%] top-[50%] z-[400] max-h-[85vh] w-[90vw] max-w-[450px] translate-x-[-50%] translate-y-[-50%] rounded-[6px] bg-neutral-200 p-[25px] shadow-[hsl(206_22%_7%_/_35%)_0px_10px_38px_-10px,_hsl(206_22%_7%_/_20%)_0px_10px_20px_-15px] focus:outline-none dark:bg-neutral-900"
      >
        <DialogTitle
          class="m-0 text-[17px] font-semibold text-neutral-800 dark:text-neutral-200"
        >
          Open File
        </DialogTitle>
        <DialogDescription
          class="mb-5 mt-[10px] text-sm leading-normal text-neutral-600 dark:text-neutral-400"
        >
          Enter the name of the file you want to open below.
        </DialogDescription>

        <ComboboxRoot
          class="relative"
          v-model="fileName"
          :default-open="true"
          :ignore-filter="
            // show ComboboxEmpty when there are no files
            myfiles.length == 0
          "
        >
          <ComboboxAnchor>
            <ComboboxInput
              class="inline-flex h-[35px] w-full flex-1 items-center justify-center rounded-lg bg-neutral-100 px-[10px] text-sm leading-none text-neutral-800 shadow-[0_0_0_1px] shadow-neutral-300 outline-none focus:shadow-[0_0_0_2px] focus:shadow-orange-400 dark:bg-neutral-800 dark:text-neutral-200 dark:shadow-neutral-700"
            />
          </ComboboxAnchor>
          <ComboboxContent class="absolute w-full">
            <ComboboxViewport
              class="z-[500] mt-1 max-h-[300px] overflow-y-auto rounded-lg bg-neutral-100 p-2 shadow-lg dark:bg-neutral-800"
            >
              <ComboboxEmpty
                class="p-2 text-sm text-neutral-600 dark:text-neutral-400"
                >No files found</ComboboxEmpty
              >
              <ComboboxItem
                v-for="file in myfiles"
                :key="file"
                :value="file"
                class="cursor-pointer rounded p-2 text-sm text-neutral-800 outline-none hover:bg-neutral-200 data-[highlighted]:bg-neutral-200 dark:text-neutral-200 dark:hover:bg-neutral-700 dark:data-[highlighted]:bg-neutral-700"
              >
                {{ file }}
              </ComboboxItem>
            </ComboboxViewport>
          </ComboboxContent>
        </ComboboxRoot>
        <div class="mt-[25px] flex justify-end gap-3">
          <DialogClose as-child>
            <button
              @click="cancel"
              class="inline-flex h-[35px] items-center justify-center rounded-lg bg-neutral-300 px-[15px] text-sm font-semibold leading-none text-neutral-800 hover:bg-neutral-400 focus:shadow-[0_0_0_2px] focus:shadow-orange-400 focus:outline-none dark:bg-neutral-800 dark:text-neutral-200 dark:hover:bg-neutral-700"
            >
              Cancel
            </button>
          </DialogClose>
          <button
            @click="myConfirm"
            :disabled="!fileName"
            class="inline-flex h-[35px] items-center justify-center rounded-lg bg-orange-500 px-[15px] text-sm font-semibold leading-none text-white hover:bg-orange-600 focus:shadow-[0_0_0_2px] focus:shadow-orange-400 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          >
            Open
          </button>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import {
  DialogRoot,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogClose,
  ComboboxRoot,
  ComboboxAnchor,
  ComboboxInput,
  ComboboxViewport,
  ComboboxEmpty,
  ComboboxItem,
  ComboboxContent,
} from 'reka-ui'

import { onMounted, ref, watch } from 'vue'
import { showFileOpenDialog } from '../state/state'
import {
  confirm,
  getOpenableFiles,
} from '../utils/query/access-manager/access-manager-web'

const fileName = ref('')

const myfiles = ref<string[]>([])
const loading = ref(true)

onMounted(async () => {
  myfiles.value = await getOpenableFiles()
  loading.value = false
})

watch(showFileOpenDialog, async (value) => {
  if (value) {
    loading.value = true
    myfiles.value = await getOpenableFiles()
    loading.value = false
  }
})

const cancel = () => {
  confirm('')
}
const myConfirm = () => {
  confirm(fileName.value)
}
</script>
