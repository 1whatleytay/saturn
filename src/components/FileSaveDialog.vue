<template>
  <DialogRoot v-model:open="showFileSaveDialog">
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
          Save File
        </DialogTitle>
        <DialogDescription
          class="mb-5 mt-[10px] text-sm leading-normal text-neutral-600 dark:text-neutral-400"
        >
          Enter a name for your file below.
        </DialogDescription>
        <fieldset class="mb-[15px] flex items-center gap-5">
          <input
            v-model="fileName"
            type="text"
            placeholder="Enter file name"
            @keyup.enter="myConfirm"
            class="inline-flex h-[35px] w-full flex-1 items-center justify-center rounded-lg bg-neutral-100 px-[10px] text-sm leading-none text-neutral-800 shadow-[0_0_0_1px] shadow-neutral-300 outline-none focus:shadow-[0_0_0_2px] focus:shadow-orange-400 dark:bg-neutral-800 dark:text-neutral-200 dark:shadow-neutral-700"
          />
        </fieldset>
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
            Save
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
} from 'reka-ui'

import { ref } from 'vue'
import { showFileSaveDialog } from '../state/state'
import { confirm } from '../utils/query/access-manager/access-manager-web'

const fileName = ref('')

const cancel = () => {
  confirm('')
}
const myConfirm = () => {
  confirm(fileName.value)
}
</script>
