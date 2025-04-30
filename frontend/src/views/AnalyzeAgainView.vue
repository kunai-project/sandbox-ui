<script lang="ts" setup>
import PageView from './PageView.vue'
import { ROUTE_NAMES } from '@/router'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import SandboxListbox from '@/components/SandboxListbox.vue'
import ToolTip from '@/components/ToolTip.vue'
import { api, apiRequest, fetchAPI } from '@/api'

const props = defineProps<{
  uuid: string // TypeScript type for UUID
}>()

const sandboxListBox = ref<InstanceType<typeof SandboxListbox> | null>(null)

const router = useRouter()

async function analyzeAgain() {
  const params = new URLSearchParams()

  if (sandboxListBox.value) {
    if (sandboxListBox.value.selectedSandbox) {
      params.append('sandbox', sandboxListBox.value.selectedSandbox.name)
    }
  }

  const uuid_opt = await fetchAPI<string>(
    apiRequest(api.endpoints.analyzeAgain, { uuid: props.uuid }, params),
  )

  if (uuid_opt) {
    router.push({ name: ROUTE_NAMES.ANALYSIS, params: { uuid: uuid_opt } })
  }
}
</script>

<template>
  <PageView>
    <template v-slot:content>
      <div class="flex-col flex h-full justify-center items-center">
        <div class="flex justify-center w-full h-40">
          <img src="@/assets/logo.svg" class="mb-4 block" />
        </div>

        <p class="text-center text-xl w-full pt-4">Select re-analysis settings for</p>

        <a
          class="pt-4"
          :href="router.resolve({ name: ROUTE_NAMES.ANALYSIS, params: { uuid: props.uuid } }).href"
          >{{ props.uuid }}</a
        >

        <div class="pt-4 flex justify-center">
          <ToolTip tip="Select sandbox" position="right">
            <template v-slot:content>
              <SandboxListbox ref="sandboxListBox" class="w-xs" />
            </template>
          </ToolTip>
        </div>

        <div class="flex w-xs items-center justify-center pt-3">
          <div class="px-3"></div>
          <button
            @click="analyzeAgain"
            class="font-medium w-1/2 h-full py-4 px-2 rounded-xl btn-primary"
          >
            Re-analyze
          </button>
        </div>
      </div>
    </template>
  </PageView>
</template>
