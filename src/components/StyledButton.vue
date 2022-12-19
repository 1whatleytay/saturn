<template>
  <button class="
    bg-neutral-700
    hover:bg-neutral-600
    px-2
    py-1
    transition-color
    duration-200
    rounded-lg
    font-bold
    flex
    items-center
  ">
    <slot />
  </button>
</template>

<script lang="ts">
export default {
  name: 'StyledButton'
}
</script>
