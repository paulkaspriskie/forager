import * as esbuild from "esbuild";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { watchLogger } from "./logger.mjs";


const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  ".."
);

const outputDir = path.resolve(rootDir, "../..", "dist");
const isWatchMode = process.argv.includes("--watch");

/** @type {esbuild.BuildOptions} */
const options = {
  absWorkingDir: rootDir,
  entryPoints: ["src/index.ts"],
  outdir: outputDir,
  bundle: true,
  format: "esm",
  platform: "browser",
  target: "es2022",
  sourcemap: true,
  minify: !isWatchMode,
  jsx: "automatic",
  plugins: [watchLogger],
};

async function build() {
  if (isWatchMode) {
    const ctx = await esbuild.context(options);
    await ctx.watch();

    console.log("Watching for changes...");
  } else {
    await esbuild.build(options);
    console.log("Build complete.");
  }
}

build().catch((error) => {
  console.error(error);
  process.exit(1);
});