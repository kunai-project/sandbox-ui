<script lang="ts" setup>
defineProps<{
  name: string
}>()
</script>

<template>
  <tr class="flex">
    <td class="text-lg pr-5 w-50">
      {{
        name
          .toString()
          .replace(/_/g, ' ')
          .replace(/\b\w/g, (c) => c.toUpperCase())
      }}
    </td>

    <td class="text max-w-xl break-all">
      <slot name="data"></slot>
    </td>
  </tr>
</template>
