<script setup lang="ts">
import { onMounted, useTemplateRef } from "vue";

const canvas = useTemplateRef("canvas");
const testPath = "C:/Users/adamj/Downloads/ilg-zkouska.png";

async function renderImage(path: string) {
  const res = await fetch(`http://image.localhost/${path}`);

  if (!res.ok) {
    console.error("Failed to load image.");
    return;
  }

  console.log(res.headers);
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
  renderImage(testPath);
});
</script>

<template>
   <canvas ref="canvas" class="w-screen h-screen"></canvas> <router-link to="/"
    >go back</router-link
  >
</template>
