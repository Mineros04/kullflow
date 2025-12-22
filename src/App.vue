<script setup lang="ts">
import ToastContainer from "@/components/ToastContainer.vue";
</script>

<template>
  <main class="overflow-hidden">
    <router-view />
  </main>
  <toast-container />
</template>
