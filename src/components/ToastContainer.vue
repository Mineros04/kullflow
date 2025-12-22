<script setup lang="ts">
import { useToast } from "@/lib/toast";

const { toasts, destroyToast } = useToast();
</script>

<template>
  <div v-if="toasts.length" class="toast toast-end toast-bottom z-50">
    <transition-group name="toast-fade">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="`alert cursor-pointer shadow-lg alert-${toast.type}`"
        @click="destroyToast(toast.id)"
      >
        <span>{{ toast.message }}</span>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: all 0.3s ease;
}
.toast-fade-enter-from,
.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(calc(var(--spacing) * 5));
}
</style>
