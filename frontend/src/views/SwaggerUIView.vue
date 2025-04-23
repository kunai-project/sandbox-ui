<script setup lang="ts">
import { onMounted } from 'vue'
import SwaggerUI from 'swagger-ui-dist/swagger-ui-es-bundle'
import 'swagger-ui-dist/swagger-ui.css'
import PageView from './PageView.vue'
import { fetchAPI } from '@/utils'

onMounted(async () => {
  let openapi = await fetchAPI<{ [key: string]: object }>('/api/openapi/json')

  if (openapi) {
    openapi.servers = [
      {
        url: `${window.location.protocol}//${window.location.hostname}:${window.location.port}`,
      },
    ]
  }

  SwaggerUI({
    dom_id: '#swagger-ui',
    //url: '/api/openapi/json', // Make sure your OpenAPI spec is accessible
    spec: openapi,
  })
})
</script>

<template>
  <PageView>
    <template v-slot:content>
      <div class="pb-10">
        <div id="swagger-ui" class="bg-white rounded-xl pb-2"></div>
      </div>
    </template>
  </PageView>
</template>
