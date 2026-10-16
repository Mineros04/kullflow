<script setup lang="ts">
import { computed, ref, useTemplateRef } from "vue";

import { Icon } from "@iconify/vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { debug, error } from "@tauri-apps/plugin-log";
import { onKeyStroke } from "@vueuse/core";

import { useToast } from "@/lib/toast";
import type { ImageVote } from "@/lib/types";

const { createToast } = useToast();

const imageCount = (history.state.imageCount as number) ?? 0;
const helpModal = useTemplateRef("helpModal");
const imageIdx = ref(0);
const imageLoading = ref(true);
const imageFailed = ref(false);
const imageUrl = computed(() =>
  convertFileSrc(String(imageIdx.value), "image")
);

function onImageError() {
  imageLoading.value = false;
  imageFailed.value = true;
  error(`Failed to load image at index ${imageIdx.value}`);
}

async function voteImage(action: ImageVote) {
  try {
    await invoke("vote_image", { index: imageIdx.value, action });
    debug(`Voted "${action}" on image at index ${imageIdx.value}`);
  } catch (err) {
    error(
      `Failed to cast "${action}" vote for image ${imageIdx.value}: ${err}`
    );
    createToast({ message: "Failed to vote for image.", type: "error" });
  }
}

onKeyStroke(["ArrowUp", "ArrowDown"], async e => {
  e.preventDefault();
  const action = e.key === "ArrowUp" ? "keep" : "delete";

  await voteImage(action);
});
</script>

<template>
  <!-- Backbround -->
  <img
    src="@/assets/bg-1.avif"
    alt=""
    class="fixed top-0 left-0 -z-1 h-screen w-screen object-cover brightness-50"
  />

  <!-- The webview decodes the original image; CSS fits it within the viewport. -->
  <div class="grid h-screen w-screen place-items-center">
    <template v-if="imageCount > 0">
      <img
        v-show="!imageLoading && !imageFailed"
        :src="imageUrl"
        alt="Current image for culling"
        class="max-h-screen max-w-full object-contain"
        @load="imageLoading = false"
        @error="onImageError"
      />
      <span
        v-if="imageLoading"
        class="loading loading-spinner loading-lg"
      ></span>
      <p v-else-if="imageFailed" role="alert">
        Unable to display this image. It may be missing, damaged, or
        unsupported.
      </p>
    </template>
    <p v-else>No images found in this directory.</p>
  </div>

  <!-- Actions dock -->
  <div class="fixed bottom-2 left-1/2 -translate-x-1/2">
    <div class="bg-base-100/20 flex gap-2 rounded-full p-4 backdrop-blur-md">
      <button
        class="btn btn-circle btn-success btn-lg ring-success ring-offset-base-100 ring-0 focus:ring-2 focus:ring-offset-2"
        @click="voteImage('keep')"
      >
        <icon icon="mdi:arrow-up-bold" class="text-2xl" />
      </button>
      <button
        class="btn btn-circle btn-error btn-lg ring-error ring-offset-base-100 ring-0 focus:ring-2 focus:ring-offset-2"
        @click="voteImage('delete')"
      >
        <icon icon="mdi:arrow-down-bold" class="text-2xl" />
      </button>
    </div>
  </div>

  <!-- Go Back button -->
  <div class="fab">
    <!-- This MUST be a div, not a button, according to DaisyUI docs. -->
    <div
      tabindex="0"
      role="button"
      class="btn btn-lg btn-circle btn-primary hover:shadow-primary/50 shadow-xl shadow-transparent transition-shadow duration-200"
    >
      <icon icon="mdi:tools" />
    </div>

    <!-- FAB inner buttons -->
    <router-link
      to="/"
      class="btn btn-lg btn-circle bg-base-100 tooltip tooltip-left"
      data-tip="Go Back"
    >
      <icon icon="mdi:arrow-u-left-top" />
    </router-link>

    <button
      class="btn btn-lg btn-circle bg-base-100 tooltip tooltip-left"
      data-tip="Help"
      @click="helpModal?.showModal()"
    >
      <icon icon="mdi:help-circle-outline" />
    </button>
  </div>

  <!-- Help modal dialog -->
  <dialog ref="helpModal" class="modal">
    <div class="modal-box">
      <h3 class="mb-4 flex items-center gap-2 text-2xl font-black">
        <icon icon="mdi:help-circle" class="inline" />
        <span>Help</span>
      </h3>
      <div class="flex flex-col gap-4">
        <h4 class="text-lg font-bold">Keyboard shortcuts</h4>
        <ul>
          <li><kbd class="kbd">ArrowUp</kbd> &ndash; keep current image</li>
          <li><kbd class="kbd">ArrowDown</kbd> &ndash; delete current image</li>
        </ul>
        <p>Alternatively, you can use the buttons in the navigation dock.</p>
        <p>
          Click outside this dialog or press <kbd class="kbd">Esc</kbd> to
          close.
        </p>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop">
      <button>close</button>
    </form>
  </dialog>
</template>
