<template>
  <div
    class="fixed z-40 h-screen w-screen bg-black bg-opacity-40 transition-opacity duration-500"
    :class="{ 'pointer-events-none opacity-0': !show }"
    @click="emit('close')"
  />

  <div v-if="show" class="pointer-events-none absolute top-12 z-50 w-full">
    <span class="pointer-events-auto">
      <slot />
    </span>
  </div>
</template>

<script setup lang="ts">
const { show } = defineProps<{ show: boolean }>()

const emit = defineEmits(['close'])
</script>
