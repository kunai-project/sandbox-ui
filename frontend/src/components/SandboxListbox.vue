<template>
  <div class="w-65" v-if="sandboxes && selectedSandbox">
    <Listbox v-model="selectedSandbox">
      <div class="relative mt-1">
        <ListboxButton
          class="relative w-full cursor-default rounded-lg bg-primary py-2 pl-3 pr-10 text-left shadow-md focus:outline-none focus-visible:border-indigo-500 focus-visible:ring-2 focus-visible:ring-white/75 focus-visible:ring-offset-2 focus-visible:ring-offset-orange-300 sm:text-sm"
        >
          <span class="block truncate"> {{ selectedSandbox.name }}</span>
          <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
            <ChevronUpDownIcon class="h-5 w-5 text-gray-400 bg-transparent" aria-hidden="true" />
          </span>
        </ListboxButton>

        <transition
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <ListboxOptions
            class="absolute top-full translate-y-2 mt-1 max-h-60 w-full overflow-auto rounded-md bg-secondary py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
          >
            <ListboxOption
              v-slot="{ active, selected }"
              v-for="sandbox in sandboxes"
              :key="sandbox.name"
              :value="sandbox"
              as="template"
            >
              <li
                :class="[
                  active ? 'bg-accent text-primary' : 'text-gray-900',
                  'relative cursor-default select-none py-2 pl-10 pr-4',
                ]"
              >
                <span :class="[selected ? 'font-medium' : 'font-normal', 'block truncate']">{{
                  sandbox.name
                }}</span>
                <span
                  v-if="selected"
                  class="absolute inset-y-0 left-0 flex items-center pl-3 text-amber-600"
                >
                  <CheckIcon class="h-5 w-5 bg-transparent text-primary" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
          </ListboxOptions>
        </transition>
      </div>
    </Listbox>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from '@headlessui/vue'
import { CheckIcon, ChevronUpDownIcon } from '@heroicons/vue/24/outline'
import { fetchAPI } from '@/utils'
import { config } from '@/config'

interface Sandbox {
  name: string
  arch: string
  kernel: string
  distribution: string
}

const sandboxes = ref<Sandbox[] | null>(null)

const selectedSandbox = ref<Sandbox | null>(null)

defineExpose({
  selectedSandbox,
})

onMounted(async () => {
  sandboxes.value = await fetchAPI(config.api.sandboxesList)
  if (sandboxes.value) {
    selectedSandbox.value = sandboxes.value[0]
  }
})
</script>
