<script setup lang="ts">
import PageView from './PageView.vue'
import { config } from '@/config'
</script>

<template>
  <PageView>
    <template v-slot:content>
      <div class="flex items-center justify-center">
        <div class="flex-col max-w-lg pt-8">
          <div>
            <p class="text-6xl py-2">About</p>
          </div>

          <div>
            <p class="text-xl py-2 text-justify">
              This service does not aim to be an additional sandboxing system competing with all the
              other existing solutions. Instead, it tries to address a very specific need:
              <b class="font-semibold">provide data directly usable by detection engineers</b>
              working with <a :href="config.kunaiLink">Kunai</a>.
            </p>

            <p class="text-4xl py-2">Goals</p>
            <ul class="list-disc text-xl pl-10">
              <li>Make detection engineers' lives easier</li>
              <li>
                Enable the community to <b class="font-semibold">share</b>
                <a :href="config.kunaiLink">Kunai</a> traces
              </li>
            </ul>

            <p class="text-4xl py-2">Non-Goals</p>
            <ul class="list-disc text-xl pl-10">
              <li>Defeat anti-sandboxing techniques</li>
              <li>Use hypervisor-based sandboxing</li>
              <li>Unpack malware samples</li>
            </ul>

            <p class="text-4xl py-2">Disclaimer</p>
            <p class="text-xl py-2 text-justify">
              Any file submission is subject to being <b class="font-semibold">disclosed</b> to
              other users of the platform and <b>shared</b> with the community. If you want to keep
              your submission private, <b class="font-semibold">no problem</b>, you can run this
              service on your own by installing
              <a :href="config.sandboxUILink">Kunai Sandbox UI</a>.
            </p>
          </div>
        </div>
      </div>
    </template>
  </PageView>
</template>
