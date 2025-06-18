<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import PageView from './PageView.vue'
import router, { ROUTE_NAMES } from '@/router'
import { useRoute } from 'vue-router'
import { apiUrl, api, fetchAPI, type AnalysesSearchResult } from '@/api'
import { ChevronRightIcon, ChevronLeftIcon } from '@heroicons/vue/24/outline'

const lastAnalyses = ref<AnalysesSearchResult | null>(null)
const initDone = ref<boolean>(false)
const pageNum = ref<number>(0)
const pageCount = ref<number>(0)
const noMoreData = ref<boolean>(false)
const searchTerm = ref<string | null>(null)
const limit: number = 10
const route = useRoute()
const hashRegex = /^(?:[0-9a-fA-F]{32}|[0-9a-fA-F]{40}|[0-9a-fA-F]{64}|[0-9a-fA-F]{128})$/

// Function to fetch data from the API
async function initialize() {
  let offset: number = 0

  const page = route.query.page as number | null

  if (page) {
    if (page > 0) {
      offset = limit * (page - 1)
    }
  }

  searchTerm.value = route.query.search as string | null

  const params = new URLSearchParams()

  if (searchTerm.value) {
    if (hashRegex.test(searchTerm.value)) {
      params.append('hash', searchTerm.value)
    } else {
      params.append('term', searchTerm.value)
    }
  }

  params.append('offset', offset.toString())
  params.append('limit', limit.toString())

  const asr = await fetchAPI<AnalysesSearchResult>(
    apiUrl(api.endpoints.analysesSearch, undefined, params),
  )

  if (!asr) {
    return
  }

  if (!asr.analyses.length && offset == 0) {
    lastAnalyses.value = null
    return
  }

  if (asr?.analyses.length > 0) {
    lastAnalyses.value = asr
  }

  // we update noMoreData
  if (asr?.offset + asr?.limit >= asr?.total) {
    noMoreData.value = true
  } else {
    noMoreData.value = false
  }

  computePageNum()

  initDone.value = true
}

function computePageNum() {
  if (lastAnalyses.value) {
    let min = Math.min(lastAnalyses.value.limit, lastAnalyses.value.analyses.length)

    pageCount.value = Math.ceil(lastAnalyses.value.total / lastAnalyses.value.limit)
    pageNum.value = Math.floor(
      (pageCount.value * (lastAnalyses.value.offset + min)) / lastAnalyses.value.total,
    )
  }
}

async function nextPage() {
  if (!noMoreData.value) {
    router.push({
      name: ROUTE_NAMES.ANALYSIS_LIST,
      query: { page: pageNum.value + 1, search: searchTerm.value },
    })
  }
}

async function previousPage() {
  if (pageNum.value > 1)
    router.push({
      name: ROUTE_NAMES.ANALYSIS_LIST,
      query: { page: pageNum.value - 1, search: searchTerm.value },
    })
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
  await initialize() // Initial fetch
})

watch(
  () => route.query,
  () => {
    initialize()
  },
)
</script>

<template>
  <PageView>
    <template v-slot:content>
      <div v-if="!lastAnalyses && initDone" class="flex justify-center h-full pt-12">
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
              <tr v-for="(item, index) in lastAnalyses.analyses" :key="index">
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

          <div class="flex justify-center pt-4">
            <div v-if="pageNum != 0 && pageCount > 1">
              <button @click="previousPage" class="rounded-lg px-2 py-2 btn-primary">
                <ChevronLeftIcon class="h-6" />
              </button>
            </div>

            <div class="px-1"></div>

            <div v-if="lastAnalyses?.analyses.length === limit && !noMoreData">
              <button @click="nextPage" class="rounded-lg px-2 py-2 btn-primary">
                <ChevronRightIcon class="h-6" />
              </button>
            </div>
          </div>

          <div class="flex justify-center pt-4 pb-20">Page {{ pageNum }} / {{ pageCount }}</div>
        </div>
      </div>
    </template>
  </PageView>
</template>
