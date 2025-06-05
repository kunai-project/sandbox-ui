<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import PageView from './PageView.vue'
import { ROUTE_NAMES } from '@/router'
import { useRoute } from 'vue-router'
import { apiUrl, api, fetchAPI, type Analysis } from '@/api'
import { ChevronRightIcon, ChevronLeftIcon } from '@heroicons/vue/24/outline'

const lastAnalyses = ref<[Analysis] | null>(null)
const pageNum = ref<number>(0)
const offset = ref<number>(0)
const noMoreData = ref<boolean>(false)
const limit: number = 10
const route = useRoute()

// Function to fetch data from the API
async function fetchData(offset: number) {
  const params = new URLSearchParams()

  params.append('offset', offset.toString())
  params.append('limit', limit.toString())

  const hash = route.query.hash as string | null
  if (hash) {
    params.append('hash', hash)
  }

  const analysis = await fetchAPI<[Analysis]>(
    apiUrl(api.endpoints.analysesSearch, undefined, params),
  )

  if (!analysis) {
    return
  }

  if (!analysis.length && offset == 0) {
    lastAnalyses.value = null
    return
  }

  if (analysis?.length > 0) {
    lastAnalyses.value = analysis
    noMoreData.value = false
  } else {
    noMoreData.value = true
  }
}

async function nextPage() {
  if (lastAnalyses.value && lastAnalyses.value.length >= limit) {
    await fetchData(offset.value + limit)
    if (!noMoreData.value) {
      offset.value += limit
      pageNum.value += 1
    }
  }
}

async function previousPage() {
  if (lastAnalyses.value) {
    offset.value -= limit
    pageNum.value -= 1
    await fetchData(offset.value)
  }
}

function reset() {
  offset.value = 0
  lastAnalyses.value = null
  pageNum.value = 0
  noMoreData.value = false
}

function convertDate(timestamp: string): string {
  const date = new Date(timestamp)

  return new Intl.DateTimeFormat('en-GB', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    timeZoneName: 'short',
    hour12: false, // Ensures 24-hour format
  }).format(date)
}

// Fetch data when the component is mounted
onMounted(async () => {
  await fetchData(offset.value) // Initial fetch
})

watch(
  () => route.query,
  () => {
    reset()
    fetchData(offset.value)
  },
)
</script>

<template>
  <PageView>
    <template v-slot:content>
      <div v-if="!lastAnalyses" class="flex justify-center h-full pt-12">
        <p class="text-2xl">No result to display</p>
      </div>
      <div v-if="lastAnalyses" class="flex justify-center h-screen pt-12 px-6">
        <div class="flex-col items-center w-full overflow-x-auto" style="scrollbar-width: none">
          <table class="w-full">
            <thead>
              <tr>
                <th class="py-5 px-4 text-left">Date</th>
                <th class="py-5 px-4 text-left">Submission Name</th>
                <th class="py-5 px-4 text-left">Status</th>
                <th class="py-5 px-4 text-left">SHA1</th>
                <th class="py-5 px-4 text-left">Analysis</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr v-for="(item, index) in lastAnalyses" :key="index">
                <td class="py-6 px-4 text-left font-semibold">{{ convertDate(item.date) }}</td>
                <td class="py-6 px-4 text-left text-wrap max-w-40 truncate">
                  {{ item.submission_name }}
                </td>
                <td class="py-6 px-4 text-left font-semibold">{{ item.status }}</td>
                <td class="py-6 px-4 text-left">
                  {{ item.sample?.sha1 }}
                </td>
                <td class="py-6 px-4 text-left">
                  <router-link :to="{ name: ROUTE_NAMES.ANALYSIS, params: { uuid: item.uuid } }">{{
                    item.uuid
                  }}</router-link>
                </td>
              </tr>
            </tbody>
          </table>

          <div class="flex justify-center pt-4 pb-20">
            <div v-if="pageNum != 0">
              <button @click="previousPage" class="rounded-lg px-2 py-2 btn-primary">
                <ChevronLeftIcon class="h-6" />
              </button>
            </div>

            <div v-if="lastAnalyses?.length === limit && !noMoreData" class="px-2">
              <button @click="nextPage" class="rounded-lg px-2 py-2 btn-primary">
                <ChevronRightIcon class="h-6" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>
  </PageView>
</template>
