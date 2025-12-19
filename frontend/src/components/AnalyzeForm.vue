<script lang="ts" setup>
import { ROUTE_NAMES } from '@/router'
import { lastAnalysisByHash } from '@/utils'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import SandboxListbox from './SandboxListbox.vue'
import ToolTip from './ToolTip.vue'
import { api, apiRequest, fetchAPI, type Analysis } from '@/api'

const sandboxListBox = ref<InstanceType<typeof SandboxListbox> | null>(null)

const lastAnalysis = ref<Analysis | null>(null)
const file = ref<File | null>(null)
const previewUrl = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const router = useRouter()

async function sha512(file: File): Promise<string> {
  const arrayBuffer = await file.arrayBuffer()
  // Compute the SHA-512 hash
  const hashBuffer = await crypto.subtle.digest('SHA-512', arrayBuffer)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')
}

async function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement
  if (target.files && target.files[0]) {
    file.value = target.files[0]
    // file.value cannot be null here
    lastAnalysis.value = await lastAnalysisByHash(await sha512(file.value))
  }
}

function goToLastAnalysis() {
  if (lastAnalysis.value) {
    router.push({ name: ROUTE_NAMES.ANALYSIS, params: { uuid: lastAnalysis.value.uuid } })
  }
}

async function postAnalysis() {
  if (!file.value) return

  const formData = new FormData()
  formData.append('file', file.value)

  if (sandboxListBox.value) {
    if (sandboxListBox.value.selectedSandbox) {
      formData.append('sandbox', sandboxListBox.value.selectedSandbox.name)
    }
  }

  const uuid = await fetchAPI<string>(
    apiRequest(api.endpoints.analyze, undefined, undefined, formData),
  )

  if (uuid) {
    router.push({ name: ROUTE_NAMES.ANALYSIS, params: { uuid: uuid } })
    resetForm()
  }
}

function resetForm() {
  file.value = null
  previewUrl.value = null
  if (fileInput.value) fileInput.value.value = ''
}
</script>

<template>
  <div class="flex-col flex justify-center items-center">
    <form @submit.prevent="postAnalysis">
      <div class="flex-col flex justify-center items-center">
        <input
          type="file"
          @change="handleFileChange"
          ref="fileInput"
          class="text-sm border border-gray-300 rounded-lg p-3 focus:outline-none w-xs"
        />

        <!-- adjust padding here so that it works with both buttons -->
        <div v-if="file" class="pt-2">
          <p><strong>Selected file:</strong> {{ file.name }}</p>
        </div>
      </div>

      <div v-if="file" class="pt-4 flex justify-center">
        <ToolTip tip="Select sandbox" position="right">
          <template v-slot:content>
            <SandboxListbox ref="sandboxListBox" class="w-xs" />
          </template>
        </ToolTip>
      </div>

      <div class="flex flex-col items-center justify-center pt-4">
        <button
          :disabled="!file"
          :hidden="!file || lastAnalysis != null"
          class="font-medium py-2 px-10 rounded-lg btn-primary"
        >
          Analyze
        </button>
      </div>
    </form>

    <div v-if="lastAnalysis" class="flex w-xs items-center justify-center pt-3">
      <button
        @click="goToLastAnalysis"
        :disabled="!lastAnalysis"
        :hidden="!lastAnalysis"
        class="font-medium w-1/2 h-full py-4 px-2 rounded-xl btn-primary"
      >
        View Last Analysis
      </button>
      <div class="px-3"></div>
      <button
        :disabled="!lastAnalysis"
        :hidden="!lastAnalysis"
        class="font-medium w-1/2 h-full py-4 px-2 rounded-xl btn-primary"
      >
        Re-analyze
      </button>
    </div>
  </div>
</template>
