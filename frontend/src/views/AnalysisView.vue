<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import PageView from './PageView.vue'
import MetadataTr from '@/components/MetadataTr.vue'
import AnalysisGraph from '@/components/AnalysisGraph.vue'
import { Switch } from '@headlessui/vue'
import ToolTip from '@/components/ToolTip.vue'
import AnalysisNotFound from '@/components/AnalysisNotFound.vue'
import { ROUTE_NAMES } from '@/router'
import { useRouter } from 'vue-router'
import { fetchAPI } from '@/utils'

const props = defineProps<{
  uuid: string // TypeScript type for UUID
}>()

const init = ref<boolean>(false)
const finished = ref<boolean>(false)
const showGraph = ref<boolean>(true)
const metadata = ref<Record<string, unknown> | null>(null)
const sandbox = ref<Record<string, string> | null>(null)
const status = ref<string | null>(null)
const dataReady = ref<boolean>(false)
const router = useRouter()

let intervalId: number | undefined

// Function to fetch data from the API
const fetchData = async () => {
  try {
    const status_res = await fetchAPI<string>(`/api/analysis/${props.uuid}/status`) // Replace with your API URL

    if (status_res) {
      status.value = status_res // Adjust based on your API response
      // if analysis is terminated or uuid is not known
      if (
        status.value == 'terminated' ||
        status.value == 'failed' ||
        status.value == 'unqueued' ||
        status.value == null
      ) {
        finished.value = true
        if (intervalId) {
          clearInterval(intervalId)
        }
      }

      if (status.value == 'terminated') {
        const metadata_res = await fetchAPI<{ [key: string]: object }>(
          `/api/analysis/${props.uuid}/metadata`,
        )

        if (metadata_res) {
          metadata.value = metadata_res
          dataReady.value = true
        }

        const sandbox_res = await fetchAPI<{ [key: string]: string }>(
          `/api/analysis/${props.uuid}/sandbox`,
        )
        if (sandbox_res) {
          sandbox.value = sandbox_res
        }
      }
    } else {
      if (intervalId) {
        clearInterval(intervalId)
      }
    }
  } catch (error) {
    console.error('Error fetching data:', error)
    if (intervalId) {
      clearInterval(intervalId)
    }
  }
}

// Fetch data when the component is mounted
onMounted(async () => {
  await fetchData() // Initial fetch
  init.value = true
  intervalId = setInterval(fetchData, 5000) // Poll every 5 seconds
})

// Clear the interval when the component is unmounted
onBeforeUnmount(() => {
  if (intervalId) {
    clearInterval(intervalId)
  }
})
</script>

<template>
  <PageView>
    <template v-slot:content>
      <!-- Analysis is not found -->
      <div class="flex items-center justify-center h-full" v-if="init && !status">
        <AnalysisNotFound />
      </div>

      <div class="pt-10 flex items-center justify-center w-full">
        <div
          v-if="init && status && !dataReady"
          class="pt-10 flex-col flex items-center justify-center w-full"
        >
          <p class="text-3xl">Status: {{ status }}</p>

          <!-- spinner -->
          <div class="pt-6">
            <div
              v-if="!finished"
              class="animate-spin rounded-full border-t-4 border-solid w-10 h-10"
            ></div>
          </div>

          <div v-if="finished && status == 'unqueued'" class="pt-4">
            <p class="text-2xl">
              There was no space in the queue you can
              <a
                :href="
                  router.resolve({ name: ROUTE_NAMES.ANALYZE_AGAIN, params: { uuid: props.uuid } })
                    .href
                "
              >
                re-analyze </a
              >later
            </p>
          </div>
        </div>

        <div v-if="dataReady" class="flex items-center">
          <p class="text-3xl">Kunai Sandbox Analysis Report</p>
          <ToolTip tip="Re-analyze file" position="bottom">
            <template v-slot:content>
              <button
                @click="
                  router.push({ name: ROUTE_NAMES.ANALYZE_AGAIN, params: { uuid: props.uuid } })
                "
                class="flex text-3xl -z-10"
              >
                <font-awesome-icon
                  icon="fa-solid fa-rotate-right"
                  class="icon pl-4 hover:text-text-hover"
                />
              </button>
            </template>
          </ToolTip>
        </div>
      </div>

      <div v-if="dataReady && metadata" class="flex justify-center items-center pt-5">
        <div class="pt-5 w-2/3">
          <div v-if="metadata">
            <p class="py-5 text-2xl text-center">Sample Information</p>
            <table class="flex w-full">
              <tbody>
                <tr v-for="(value, key) in metadata" :key="key">
                  <div v-if="value">
                    <MetadataTr :name="key" class="flex">
                      <template v-slot:data>
                        {{ value }}
                      </template>
                    </MetadataTr>
                  </div>
                </tr>
              </tbody>
            </table>
          </div>

          <div v-if="sandbox">
            <p class="py-5 text-2xl text-center">Sandbox Information</p>
            <table class="flex w-full">
              <tbody>
                <tr v-for="(value, key) in sandbox" :key="key">
                  <div v-if="value">
                    <MetadataTr :name="key" class="flex">
                      <template v-slot:data>
                        {{ value }}
                      </template>
                    </MetadataTr>
                  </div>
                </tr>
              </tbody>
            </table>
          </div>

          <div>
            <p class="py-5 text-2xl text-center">Analysis Data</p>
            <table class="flex w-full">
              <tbody>
                <MetadataTr name="VirusTotal">
                  <template v-slot:data>
                    <ToolTip tip="Check hash on VirusTotal (no submission)" position="bottom">
                      <template v-slot:content>
                        <a
                          class="font-bold"
                          :href="'https://www.virustotal.com/gui/file/' + metadata['sha256']"
                        >
                          <font-awesome-icon
                            icon="fa-solid fa-arrow-up-right-from-square"
                            size="lg"
                            class="icon"
                          />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="Pcap">
                  <template v-slot:data>
                    <ToolTip tip="Download pcap" position="bottom">
                      <template v-slot:content>
                        <a :href="`/api/analysis/${uuid}/pcap`" :download="uuid + '.pcap'">
                          <font-awesome-icon icon="fa-solid fa-download" size="lg" class="icon" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="Kunai Logs">
                  <template v-slot:data>
                    <ToolTip tip="Download Kunai logs" position="bottom">
                      <template v-slot:content>
                        <a :href="`/api/analysis/${uuid}/logs`" :download="uuid + '.jsonl.gz'">
                          <font-awesome-icon icon="fa-solid fa-download" size="lg" class="icon" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="MISP Event">
                  <template v-slot:data>
                    <ToolTip tip="Download MISP event" position="bottom">
                      <template v-slot:content>
                        <a :href="`/api/analysis/${uuid}/misp-event`" :download="uuid + '.json'">
                          <font-awesome-icon icon="fa-solid fa-download" size="lg" class="icon" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="Analysis Graph">
                  <template v-slot:data>
                    <ToolTip tip="Download Analysis Graph" position="bottom">
                      <template v-slot:content>
                        <a :href="`/api/analysis/${uuid}/graph`" :download="uuid + '.svg'">
                          <font-awesome-icon icon="fa-solid fa-download" size="lg" class="icon" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="Show Graph" class="pt-8">
                  <template v-slot:data>
                    <div class="flex">
                      <Switch
                        v-model="showGraph"
                        :class="showGraph ? 'bg-text' : 'bg-kblue-600'"
                        class="relative inline-flex h-[30px] w-[66px] shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-white/75"
                      >
                        <span class="sr-only text">Use setting</span>
                        <span
                          aria-hidden="true"
                          :class="showGraph ? 'translate-x-9' : 'translate-x-0'"
                          class="pointer-events-none inline-block h-[26px] w-[26px] transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out"
                        />
                      </Switch>
                    </div>
                  </template>
                </MetadataTr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <div v-if="dataReady" :hidden="!showGraph" class="flex justify-center py-10">
        <AnalysisGraph class="flex w-2/3 aspect-[3/2]" :svgUrl="`/api/analysis/${uuid}/graph`" />
      </div>
    </template>
  </PageView>
</template>
