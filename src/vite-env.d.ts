/** biome-ignore-all lint/complexity/noBannedTypes: false positive */
/** biome-ignore-all lint/suspicious/noExplicitAny: any is allowed */
/// <reference types="vite/client" />
/// <reference types="unplugin-vue-router/client" />

declare module "*.vue" {
	import type { DefineComponent } from "vue";
	const component: DefineComponent<{}, {}, any>;
	export default component;
}

declare module "@fontsource/*";
