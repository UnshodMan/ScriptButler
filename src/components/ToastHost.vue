<script setup lang="ts">
import { toasts } from '../lib/toast'
</script>

<template>
  <div class="toast-host">
    <div v-for="t in toasts" :key="t.id" class="toast" :class="t.kind">
      {{ t.text }}
    </div>
  </div>
</template>
