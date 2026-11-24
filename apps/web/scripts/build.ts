import * as esbuild from "esbuild";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { htmlPlugin } from "./plugins/html.js";
import { postcssPlugin } from "./plugins/postcss.js";
import { watchLogger } from "./plugins/logger.js";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  ".."
);

const outputDir = path.resolve(rootDir, "../..", "dist");

const isWatchMode = process.argv.includes("--watch");
const isServeMode = process.argv.includes("--serve");

const options: esbuild.BuildOptions = {
  absWorkingDir: rootDir,
  entryPoints: ["src/index.ts"],
  outdir: outputDir,
  bundle: true,
  format: "esm",
  platform: "browser",
  sourcemap: isWatchMode,
  minify: !isWatchMode,
  metafile: true,
  plugins: [
    htmlPlugin({
      template: "templates/index.html",
    }),
    postcssPlugin,
    watchLogger,
  ],
};

async function build() {
  if (isServeMode) {
    const ctx = await esbuild.context(options);

    await ctx.watch();

    const server = await ctx.serve({
      host: "127.0.0.1",
      port: 3000,
      servedir: outputDir,
    });

    console.log(
      `Development server running at http://127.0.0.1:3000`
    );
    
  } else if (isWatchMode) {
    const ctx = await esbuild.context(options);

    await ctx.watch();

    console.log("Watching for changes...");
  } else {
    await esbuild.build(options);

    console.log("Build complete.");
  }
}

build().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
