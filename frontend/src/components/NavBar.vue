<script lang="ts" setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { config } from '@/config.ts'
import ToolTip from '@/components/ToolTip.vue'
import { ROUTE_NAMES } from '@/router'

const search = ref<string | null>(null)
const router = useRouter()

function handleInputChange(event: Event) {
  const target = event.target as HTMLInputElement
  search.value = target.value
}

async function handleEnter() {
  router.push({ name: ROUTE_NAMES.ANALYSIS_LIST, query: { hash: search.value } })
  search.value = null
}
</script>

<template>
  <nav class="bg-background-hard left-0 w-screen fixed z-50">
    <div class="flex items-center h-14 w-full">
      <!-- Set a larger height here -->
      <!-- Logo Section -->
      <div class="pl-4 flex">
        <router-link to="/"><img src="@/assets/logo.svg" class="h-10" alt="Logo" /></router-link>
      </div>

      <!-- Search Input Section -->
      <div class="flex w-128 px-4">
        <input
          @change="handleInputChange"
          @keyup.enter="handleEnter"
          v-model="search"
          type="text"
          placeholder="Search by hash"
          class="w-full max-w-md h-10 px-2 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-background"
        />
      </div>

      <!-- fill space in the middle -->
      <div class="flex flex-grow"></div>

      <div class="flex h-full items-center pr-4">
        <ToolTip tip="All Analysis" position="bottom">
          <template v-slot:content>
            <router-link :to="{ name: ROUTE_NAMES.ANALYSIS_LIST }">
              <font-awesome-icon
                icon="fa-solid fa-bars"
                class="icon text-text hover:text-text-hover"
                size="2xl"
              />
            </router-link>
          </template>
        </ToolTip>
      </div>

      <div class="flex h-full items-center pr-4">
        <ToolTip tip="API Documentation" position="bottom">
          <template v-slot:content>
            <router-link :to="{ name: ROUTE_NAMES.SWAGGER_UI }">
              <font-awesome-icon
                :icon="['fac', 'swagger']"
                class="icon text-text hover:text-text-hover"
                size="2xl"
              />
            </router-link>
          </template>
        </ToolTip>
      </div>

      <div class="pr-4 text-xl">
        <ToolTip tip="About" position="bottom">
          <template v-slot:content>
            <router-link to="/about">
              <font-awesome-icon
                icon="fa-solid fa-question"
                class="icon text-text hover:text-text-hover"
                size="xl"
              />
            </router-link>
          </template>
        </ToolTip>
      </div>

      <div class="pr-8">
        <a :href="config.sandboxUILink"
          ><font-awesome-icon
            icon="fa-brands fa-github"
            size="2xl"
            class="icon text-text hover:text-text-hover"
        /></a>
      </div>
    </div>
  </nav>
</template>
