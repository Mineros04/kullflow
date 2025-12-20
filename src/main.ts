import { createApp } from "vue";
import App from "./App.vue";
// Styles
import "./style.css";
// Routing
import { createRouter, createWebHashHistory } from "vue-router";
import { routes, handleHotUpdate } from "vue-router/auto-routes";
// Fonts
import "@fontsource/poppins/400";
import "@fontsource/poppins/600";
import "@fontsource/poppins/700";
import "@fontsource/poppins/900";

// Automatic routing.
const router = createRouter({
  history: createWebHashHistory(),
  routes
});

// HMR for page navigation.
if (import.meta.hot) {
  handleHotUpdate(router);
}

const app = createApp(App);

app.use(router);

app.mount("#app");
