<script setup lang="ts">
import NavBar from '@/components/NavBar.vue'
</script>

<template>
  <NavBar />

  <div class="pt-14 h-[calc(100vh-12rem)]">
    <slot name="content"></slot>
  </div>
</template>
