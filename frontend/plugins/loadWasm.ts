import { Plugin, PluginMiddleware } from "$fresh/server.ts";
import * as path from "$std/path/mod.ts";

export default function loadWasm(wasmPath: string): Plugin {
  const projectDir = Deno.cwd();
  const middlewares: Plugin["middlewares"] = [];

  const wasmMiddleware: PluginMiddleware = {
    path: "/",
    middleware: {
      handler: async (_req, ctx) => {
        const pathname = ctx.url.pathname;
        if (!pathname.endsWith(".wasm")) {
          return ctx.next();
        }

        const filename = path.join(
          projectDir,
          wasmPath,
          path.basename(pathname),
        );
        let content = new Uint8Array();
        try {
          content = await Deno.readFile(filename);
        } catch (err) {
          if (err instanceof Deno.errors.NotFound) {
            return ctx.next();
          }
          console.error(err);
        }

        return new Response(content, {
          status: 200,
          headers: {
            "Content-Type": "application/wasm",
            "Cache-Control": "no-cache, no-store, max-age=0, must-revalidate",
          },
        });
      },
    },
  };

  return {
    name: "loadWasm",
    configResolved: (config) => {
      if (config.dev) {
        middlewares.push(wasmMiddleware);
      }
    },
    middlewares,
  };
}
