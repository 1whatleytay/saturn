<template>
  <div class="p-12">
    <div class="text-4xl text-gray-100 text-white text-light flex items-center">
      <RocketLaunchIcon class="w-7 h-7 mr-2" /> Saturn
    </div>

    <div class="flex">
      <div class="text-sm md:w-1/2">
        <div class="mt-8 mb-4 font-heavy text-xs">
          Get Started
        </div>

        <input ref="file" type="file" class="hidden" @change="interpret" />

        <StyledButton @click="pick">
          <PlayIcon class="w-4 h-4 mr-2" />
          Interpret
        </StyledButton>


        <StyledButton @click="pick">
          <WrenchIcon class="w-4 h-4 mr-2" />
          Develop
        </StyledButton>
      </div>

      <div class="w-1/2 hidden md:block">
        <div class="mt-8 font-heavy text-xs">
          <div class="mb-4">
            Recent
          </div>

          <div class="text-gray-300">
            Nothing yet.
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { PlayIcon, WrenchIcon, RocketLaunchIcon } from '@heroicons/vue/24/solid'
import { useRouter } from 'vue-router'

import StyledButton from '../components/StyledButton.vue'

export default defineComponent({
  name: 'GettingStarted',

  components: { StyledButton, PlayIcon, WrenchIcon, RocketLaunchIcon },

  setup() {
    const router = useRouter()

    const file = ref(null as HTMLInputElement | null)

    function select(file: File) {
      const name = file.name
      const reader = new FileReader()
      reader.readAsArrayBuffer(file)

      reader.addEventListener('load', () => {
        console.log(reader.result)

        // state.value = { name, blob: reader.result as ArrayBuffer }

        router.push('/interpret')
      })
    }

    return {
      pick() {
        file.value?.click()
      },

      interpret() {
        const files = file.value?.files

        if (files && files.length != 0) {
          select(files[0])
        }
      },

      file
    }
  }
})
</script>
