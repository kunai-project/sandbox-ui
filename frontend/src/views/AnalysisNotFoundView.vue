<script setup lang="ts">
import PageView from './PageView.vue'
import AnalysisNotFound from '@/components/AnalysisNotFound.vue'
</script>

<template>
  <PageView>
    <template v-slot:content>
      <!-- Analysis is not found -->
      <div class="flex items-center justify-center h-full">
        <AnalysisNotFound />
      </div>
    </template>
  </PageView>
</template>
