import { Plugin, PluginMiddleware } from "$fresh/server.ts";
import * as path from "$std/path/mod.ts";
import * as fs from "$std/fs/mod.ts";

export default function loadWasm(wasmPath: string): Plugin {
  const projectDir = Deno.cwd();
  const middlewares: Plugin["middlewares"] = [];

  const devWasmMiddleware: PluginMiddleware = {
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

  const productionWasmMiddleware: PluginMiddleware = {
    path: "/_frsh",
    middleware: {
      handler: (_req, ctx) => {
        const filename = path.basename(ctx.url.pathname);
        if (filename.endsWith(".wasm")) {
          return new Response(null, {
            status: 308,
            headers: { location: `/${filename}` },
          });
        }
        return ctx.next();
      },
    },
  };

  return {
    name: "loadWasm",
    configResolved: (config) => {
      middlewares.push(
        config.dev ? devWasmMiddleware : productionWasmMiddleware,
      );
    },
    middlewares,
    async buildStart(config) {
      const outDir = path.join(config.build.outDir, "static");
      const wasmDir = path.join(projectDir, wasmPath);
      const wasmFiles = fs.walk(wasmDir, {
        exts: ["wasm"],
        includeDirs: false,
        includeFiles: true,
      });
      for await (const file of wasmFiles) {
        const relFilePath = path.relative(wasmDir, file.path);
        const outPath = path.join(outDir, relFilePath);
        await Deno.mkdir(path.dirname(outPath), { recursive: true });
        await Deno.copyFile(file.path, outPath);
      }
    },
  };
}
