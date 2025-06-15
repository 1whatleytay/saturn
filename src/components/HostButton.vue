<template>
  <ToastProvider>
    <DialogRoot v-model:open="joinTabOpen">
      <PopoverRoot>
        <PopoverTrigger>
          <button
            class="flex h-10 w-10 shrink-0 items-center justify-center font-black text-slate-800 hover:bg-slate-300 dark:text-slate-300 dark:hover:bg-slate-800"
          >
            <UserPlusIcon class="h-4 w-4" />
          </button>
        </PopoverTrigger>
        <PopoverContent
          class="ml-2 min-w-60 gap-2 rounded-lg border bg-neutral-200 p-2 text-sm text-slate-800 shadow-md dark:border-neutral-700 dark:bg-neutral-900 dark:text-slate-300"
        >
          <div class="flex w-full">
            <button
              class="flex items-center rounded p-2 text-left hover:bg-neutral-300 disabled:cursor-not-allowed disabled:hover:bg-inherit dark:hover:bg-neutral-700"
              @click="handleClick()"
              :disabled="!tabsState.tabs.length"
              v-if="tab()?.yjs == undefined"
            >
              Host selected tab
            </button>
            <button
              class="flex items-center rounded p-2 text-left hover:bg-neutral-300 disabled:cursor-not-allowed disabled:hover:bg-inherit dark:hover:bg-neutral-700"
              @click="showUuid()"
              v-else
            >
              Copy join link
            </button>
            <DialogTrigger
              class="ml-auto flex items-center rounded p-2 text-left hover:bg-neutral-300 dark:hover:bg-neutral-700"
            >
              Join from code
            </DialogTrigger>
          </div>
          <div
            class="my-2 border-t border-neutral-300 dark:border-neutral-700"
            v-if="tabsState.tabs.length"
          />
          <div class="flex w-full items-center rounded-r p-1 text-left">
            {{ tab()!.title }}

            <span
              v-if="isSyncing(tab()!.uuid)"
              class="ml-auto mr-1 h-2 w-2 rounded-full bg-green-500"
            ></span>
          </div>
        </PopoverContent>
      </PopoverRoot>

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
            Join Remote Editor
          </DialogTitle>
          <DialogDescription
            class="mb-5 mt-[10px] text-sm leading-normal text-neutral-600 dark:text-neutral-400"
          >
            Enter a name for your file below.
          </DialogDescription>
          <fieldset class="mb-[15px] flex items-center gap-5">
            <input
              v-model="joinTabStr"
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
              :disabled="!joinTabStr"
              class="inline-flex h-[35px] items-center justify-center rounded-lg bg-orange-500 px-[15px] text-sm font-semibold leading-none text-white hover:bg-orange-600 focus:shadow-[0_0_0_2px] focus:shadow-orange-400 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
            >
              Save
            </button>
          </div>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>

    <ToastRoot
      v-model:open="open"
      class="flex flex-col gap-x-[15px] rounded-lg border bg-white p-[15px] shadow-sm data-[swipe=cancel]:translate-x-0 data-[swipe=move]:translate-x-[var(--reka-toast-swipe-move-x)] data-[state=closed]:animate-hide data-[state=open]:animate-slideIn data-[swipe=end]:animate-swipeOut data-[swipe=cancel]:transition-[transform_200ms_ease-out] dark:border-neutral-700 dark:bg-neutral-900"
    >
      <ToastTitle class="mb-[5px] text-sm font-medium">Copied</ToastTitle>
      <ToastDescription as-child>
        <span>
          Copied join id <code>{{ tabId }}</code> to clipboard
        </span>
      </ToastDescription>
    </ToastRoot>
    <ToastViewport
      class="fixed bottom-0 right-0 z-[2147483647] m-0 flex w-[390px] max-w-[100vw] list-none flex-col gap-[10px] p-[var(--viewport-padding)] outline-none [--viewport-padding:_25px]"
    />
  </ToastProvider>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { tab, tabsState } from '../state/state'
import {
  ToastProvider,
  ToastRoot,
  ToastTitle,
  ToastDescription,
  ToastViewport,
  PopoverRoot,
  PopoverTrigger,
  PopoverContent,
  DialogRoot,
  DialogPortal,
  DialogClose,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogTrigger,
} from 'reka-ui'
import { UserPlusIcon } from '@heroicons/vue/24/solid'
import { host, isSyncing, join } from '../utils/codemirror/collab'

const joinTabOpen = ref(false)
const joinTabStr = ref('')

const open = ref(false)
const tabId = ref('')

const timerRef = ref(0)

async function handleClick() {
  const uuid = tab()?.uuid
  if (!uuid) {
    return
  }

  const success = host()
  if (success) {
    showUuid()
  }
}

async function showUuid() {
  const uuid = tab()?.uuid
  if (!uuid) {
    return
  }

  open.value = true
  tabId.value = uuid
  window.clearTimeout(timerRef.value)

  await navigator.clipboard.writeText(uuid)

  timerRef.value = window.setTimeout(() => {
    open.value = false
  }, 5000)
}

function myConfirm() {
  if (joinTabStr.value) {
    join(joinTabStr.value)
    joinTabStr.value = ''
  }
  joinTabOpen.value = false
}

function cancel() {
  joinTabStr.value = ''
}
</script>
