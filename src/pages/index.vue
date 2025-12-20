<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { open } from "@tauri-apps/plugin-dialog";

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
}
</script>

<template>

  <div>
     <img
      src="@/assets/bg-1.avif"
      alt=""
      class="fixed top-0 left-0 w-screen h-screen object-cover -z-1"
    />
    <div class="h-screen w-screen grid place-items-center">

      <div
        class="p-16 card bg-base-300 shadow-glow-card rounded-4xl flex flex-col items-center gap-4"
      >

        <h1 class="card-title text-4xl font-black tracking-wide"> KullFlow </h1>
         <span> Begin by choosing a directory to cull. </span> <router-link
          class="btn btn-info text-xl mt-4"
          to="/cull"
          > <Icon icon="mdi:folder-open" /> <span>Choose a directory</span>
          </router-link
        >
      </div>

    </div>

  </div>

</template>

