import { defineConfig } from "$fresh/server.ts";
import tailwind from "$fresh/plugins/tailwind.ts";
import loadWasm from "#plugins/loadWasm.ts";

export default defineConfig({
  plugins: [tailwind(), loadWasm("wasm")],
});
