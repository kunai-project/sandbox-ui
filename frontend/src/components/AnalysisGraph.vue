<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import SvgPanZoom from 'svg-pan-zoom'
import svgPanZoom from 'svg-pan-zoom'
import { ArrowsPointingOutIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{ svgUrl: string }>()
const svgContainer = ref<HTMLElement | null>(null)
const svgContent = ref<string | null>(null)
const panZoomInstance = ref<SvgPanZoom.Instance | null>(null)

const fetchSvg = async () => {
  try {
    if (svgContent.value) {
      return
    }
    const response = await fetch(props.svgUrl)

    if (!response.ok) throw new Error('Failed to load SVG')
    svgContent.value = await response.text()

    // Wait for the SVG to be rendered in the DOM
    await nextTick()

    // Initialize svg-pan-zoom after the SVG is properly injected
    initializeSvgPanZoom()
  } catch (error) {
    console.error('Error fetching SVG:', error)
  }
}

const initializeSvgPanZoom = () => {
  if (!svgContainer.value) return

  const svgElement = svgContainer.value.querySelector('svg')
  if (svgElement) {
    panZoomInstance.value = svgPanZoom(svgElement, {
      zoomEnabled: true,
      controlIconsEnabled: false,
      fit: false,
      center: true,
      minZoom: 0.1,
      maxZoom: 2,
    })
  } else {
    console.error('SVG element not found inside container.')
  }
}

onMounted(fetchSvg)

// Watch for changes in `svgUrl` and re-fetch the SVG
watch(() => props.svgUrl, fetchSvg)
</script>

<template>
  <div class="flex relative">
    <div class="absolute right-2 top-2 pl-2 flex items-center">
      <button
        class="h-11 w-11 px-2 py-2 rounded-full font-medium bg-background-hard hover:bg-background"
        @click="panZoomInstance?.fit() && panZoomInstance.center()"
      >
        <ArrowsPointingOutIcon class="" />
      </button>
    </div>

    <div ref="svgContainer" class="flex-col rounded-2xl w-full h-full bg-accent">
      <svg v-html="svgContent" class="svg-wrapper rounded-2xl w-full h-full"></svg>
    </div>
  </div>
</template>
