<script setup lang="ts">
import ToastProvider from "@/components/ToastProvider.vue";
</script>

<template>
  <main class="overflow-hidden">
    <router-view />
  </main>
  <toast-provider />
</template>
