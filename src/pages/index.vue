<script setup lang="ts">
import { useRouter } from "vue-router";

import { Icon } from "@iconify/vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const router = useRouter();

async function selectDirectory() {
  const dir = await open({
    directory: true,
    multiple: false,
    title: "Select a Directory to Cull"
  });

  if (dir == null) {
    console.log("dir open cancelled");
    return;
  }

  // Directory provided, pass it to backend to parse and prepare state.
  const imageCount = (await invoke("init_images", { dirStr: dir })) as number;
  router.push({
    path: "/cull",
    query: { imageCount }
  });
}
</script>

<template>
  <img
    src="@/assets/bg-1.avif"
    alt=""
    class="fixed top-0 left-0 -z-1 h-screen w-screen object-cover"
  />
  <div class="grid h-screen w-screen place-items-center">
    <div
      class="card bg-base-300 shadow-glow-card flex flex-col items-center gap-4 rounded-4xl p-16"
    >
      <h1 class="card-title text-4xl font-black tracking-wide">KullFlow</h1>
      <span>Begin by choosing a directory to cull.</span>
      <button class="btn btn-info mt-4 text-xl" @click="selectDirectory">
        <Icon icon="mdi:folder-open" /> <span>Choose a directory</span>
      </button>
    </div>
  </div>
</template>
