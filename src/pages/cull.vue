<script setup lang="ts">
import { onMounted, useTemplateRef } from "vue";

import { Icon } from "@iconify/vue";

const canvas = useTemplateRef("canvas");

async function renderImage(idx: number) {
  const res = await fetch(`http://image.localhost/${idx}`);

  if (!res.ok) {
    console.error("Failed to load image.");
    return;
  }

  const width = parseInt(res.headers.get("X-Image-Width")!);
  const height = parseInt(res.headers.get("X-Image-Height")!);

  const buffer = await res.arrayBuffer();
  const ctx = canvas.value!.getContext("2d")!;

  canvas.value!.width = width;
  canvas.value!.height = height;

  const pixelData = new Uint8ClampedArray(buffer);
  const imageData = new ImageData(pixelData, width, height);

  ctx.putImageData(imageData, 0, 0);
}

onMounted(() => {
  renderImage(0);
});
</script>

<template>
  <!-- Backbround -->
  <img
    src="@/assets/bg-1.avif"
    alt=""
    class="fixed top-0 left-0 -z-1 h-screen w-screen object-cover brightness-50"
  />

  <!-- Image rendering canvas -->
  <div class="grid h-screen w-screen place-items-center">
    <canvas ref="canvas" class="h-screen"></canvas>
  </div>

  <!-- Actions dock -->
  <div class="fixed bottom-2 left-1/2 -translate-x-1/2">
    <div class="bg-base-100/20 flex gap-2 rounded-full p-4 backdrop-blur-md">
      <button
        class="btn btn-circle btn-success btn-lg ring-success ring-offset-base-100 ring-0 focus:ring-2 focus:ring-offset-2"
      >
        <icon icon="mdi:arrow-up-bold" class="text-2xl" />
      </button>
      <button
        class="btn btn-circle btn-error btn-lg ring-error ring-offset-base-100 ring-0 focus:ring-2 focus:ring-offset-2"
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
      onclick="help_modal.showModal()"
    >
      <icon icon="mdi:help-circle-outline" />
    </button>
  </div>

  <!-- Help modal dialog -->
  <dialog id="help_modal" class="modal">
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
