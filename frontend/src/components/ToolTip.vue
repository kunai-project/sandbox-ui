<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  tip: string
  position: string
}>()

// Reactive state
const showTooltip = ref(false)

function isBottom(): boolean {
  return props.position == 'bottom' || props.position == 'b'
}

function isRight(): boolean {
  return props.position == 'right' || props.position == 'r'
}

function isLeft(): boolean {
  return props.position == 'left' || props.position == 'l'
}
</script>

<template>
  <div class="relative inline-block">
    <!-- Button -->
    <div @mouseenter="showTooltip = true" @mouseleave="showTooltip = false">
      <slot name="content"></slot>
    </div>

    <!-- Tooltip (positioned below button) -->
    <div
      v-if="showTooltip && isBottom()"
      class="absolute text-center left-1/2 -translate-x-1/2 translate-y-2 mt-2 bg-gray-700 text-white text-sm px-3 py-2 rounded-md shadow-lg transition-opacity duration-200 w-auto whitespace-nowrap z-100"
    >
      {{ props.tip }}
      <!-- Tooltip Arrow -->
      <div
        class="absolute left-1/2 -translate-x-1/2 -top-1.5 border-4 border-transparent border-b-gray-700"
      ></div>
    </div>

    <!-- right side -->
    <div
      v-if="showTooltip && isRight()"
      class="absolute text-center left-full -translate-y-full translate-x-2 ml-2 bg-gray-700 text-white text-sm px-3 py-2 rounded-md shadow-lg transition-opacity duration-200 w-auto whitespace-nowrap z-100"
    >
      {{ props.tip }}
      <!-- Tooltip Arrow -->
      <div
        class="absolute -left-1.5 top-1/2 -translate-y-1/2 border-4 border-transparent border-r-gray-700"
      ></div>
    </div>

    <!-- left side -->
    <div
      v-if="showTooltip && isLeft()"
      class="absolute text-center right-full -translate-y-full -translate-x-2 ml-2 bg-gray-700 text-white text-sm px-3 py-2 rounded-md shadow-lg transition-opacity duration-200 w-auto whitespace-nowrap z-100"
    >
      {{ props.tip }}
      <!-- Tooltip Arrow -->
      <div
        class="absolute -right-1.5 top-1/2 -translate-y-1/2 border-4 border-transparent border-l-gray-700"
      ></div>
    </div>
  </div>
</template>
