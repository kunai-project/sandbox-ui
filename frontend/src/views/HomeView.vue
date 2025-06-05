<script setup lang="ts">
import FileUpload from '@/components/AnalyzeForm.vue'
import PageView from './PageView.vue'
import { config } from '@/config'
import IconKunai from '@/components/icons/IconKunai.vue'
</script>

<template>
  <PageView>
    <template v-slot:content>
      <div class="flex items-center justify-center h-full pt-12">
        <div class="flex-col items-center">
          <div class="flex justify-center w-full h-40">
            <IconKunai></IconKunai>
          </div>

          <p class="text-center text-xl w-full pt-4">
            This service allows you to run <br />
            Linux malware samples in a sandbox <br />
            and monitor it with <a :href="config.kunaiLink">Kunai</a>
          </p>

          <div class="flex justify-center w-full pt-4">
            <FileUpload />
          </div>
        </div>
      </div>
    </template>
  </PageView>
</template>
