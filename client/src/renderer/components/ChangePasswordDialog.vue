<script setup lang="ts">
defineProps<{ visible: boolean }>()
defineEmits<{ 'update:visible': [value: boolean] }>()
</script>

<template>
  <div v-if="visible" />
</template>
