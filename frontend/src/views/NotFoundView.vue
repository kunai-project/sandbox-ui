<script setup lang="ts"></script>

<template>
  <div class="h-screen flex flex-col items-center justify-center">
    <h1 class="text-4xl font-bold mb-4">404 - Page Not Found</h1>
    <p class="text-white-600 mb-6">Oops! The page you're looking for doesn't exist.</p>
    <div class="py-4">
      <router-link to="/" class="bg-kblue text-white px-4 py-2 rounded-lg hover:bg-kblue-hover">
        Go Back Home
      </router-link>
    </div>
  </div>
</template>
