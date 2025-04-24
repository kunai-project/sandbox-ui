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
import { AnalysisStatus, api, apiUrl, fetchAPI } from '@/api'
import {
  ArrowDownTrayIcon,
  ArrowPathRoundedSquareIcon,
  ArrowTopRightOnSquareIcon,
} from '@heroicons/vue/24/outline'

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
    const path_params = { uuid: props.uuid }
    const status_res = await fetchAPI<AnalysisStatus>(
      apiUrl(api.endpoints.analysisStatus, path_params, undefined),
    ) // Replace with your API URL

    if (status_res) {
      status.value = status_res // Adjust based on your API response
      // if analysis is terminated or uuid is not known
      if (
        status.value == AnalysisStatus.Terminated ||
        status.value == AnalysisStatus.Failed ||
        status.value == AnalysisStatus.Unqueued ||
        status.value == null
      ) {
        finished.value = true
        if (intervalId) {
          clearInterval(intervalId)
        }
      }

      if (status.value == AnalysisStatus.Terminated) {
        const metadata_res = await fetchAPI<{ [key: string]: object }>(
          apiUrl(api.endpoints.analysisMetadata, path_params),
        )

        if (metadata_res) {
          metadata.value = metadata_res
          dataReady.value = true
        }

        const sandbox_res = await fetchAPI<{ [key: string]: string }>(
          apiUrl(api.endpoints.analysisSandbox, path_params),
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

          <div v-if="finished && status == AnalysisStatus.Unqueued" class="pt-4">
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
          <p class="text-3xl flex items-center">
            Kunai Sandbox Analysis Report &nbsp;

            <ToolTip tip="Re-analyze file" position="bottom">
              <template v-slot:content>
                <button
                  @click="
                    router.push({ name: ROUTE_NAMES.ANALYZE_AGAIN, params: { uuid: props.uuid } })
                  "
                  class="h-full flex"
                >
                  <ArrowPathRoundedSquareIcon class="h-10" />
                </button>
              </template>
            </ToolTip>
          </p>
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
                          <ArrowTopRightOnSquareIcon class="h-6" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="Pcap">
                  <template v-slot:data>
                    <ToolTip tip="Download pcap" position="bottom">
                      <template v-slot:content>
                        <a
                          :href="
                            apiUrl(api.endpoints.analysisPcap, {
                              uuid: props.uuid,
                            })
                          "
                          :download="uuid + '.pcap'"
                        >
                          <ArrowDownTrayIcon class="h-6" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="Kunai Logs">
                  <template v-slot:data>
                    <ToolTip tip="Download Kunai logs" position="bottom">
                      <template v-slot:content>
                        <a
                          :href="apiUrl(api.endpoints.analysisLogs, { uuid: props.uuid })"
                          :download="uuid + '.jsonl.gz'"
                        >
                          <ArrowDownTrayIcon class="h-6" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="MISP Event">
                  <template v-slot:data>
                    <ToolTip tip="Download MISP event" position="bottom">
                      <template v-slot:content>
                        <a
                          :href="apiUrl(api.endpoints.analysisMispEvent, { uuid: props.uuid })"
                          :download="uuid + '.json'"
                        >
                          <ArrowDownTrayIcon class="h-6" />
                        </a>
                      </template>
                    </ToolTip>
                  </template>
                </MetadataTr>

                <MetadataTr name="Analysis Graph">
                  <template v-slot:data>
                    <ToolTip tip="Download Analysis Graph" position="bottom">
                      <template v-slot:content>
                        <a
                          :href="apiUrl(api.endpoints.analysisGraph, { uuid: props.uuid })"
                          :download="uuid + '.svg'"
                        >
                          <ArrowDownTrayIcon class="h-6" />
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
                        :class="showGraph ? 'bg-text' : 'bg-primary'"
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
        <AnalysisGraph
          class="flex w-2/3 aspect-[3/2]"
          :svgUrl="apiUrl(api.endpoints.analysisGraph, { uuid: props.uuid })"
        />
      </div>
    </template>
  </PageView>
</template>
