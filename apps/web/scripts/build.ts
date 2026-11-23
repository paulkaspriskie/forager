import * as esbuild from "esbuild";
import path from "node:path";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import postcss from "postcss";
import autoprefixer from "autoprefixer";
import postcssNesting from "postcss-nesting";
import { watchLogger } from "./plugins/logger.js";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  ".."
);

const outputDir = path.resolve(rootDir, "../..", "dist");
const isWatchMode = process.argv.includes("--watch");

// PostCSS processor
const postcssProcessor = postcss([
  postcssNesting(),
  autoprefixer(),
]);

// esbuild PostCSS plugin
const postcssPlugin: esbuild.Plugin = {
  name: "postcss",

  setup(build) {
    build.onLoad({ filter: /\.css$/ }, async (args) => {
      const source = await readFile(args.path, "utf8");

      const result = await postcssProcessor.process(source, {
        from: args.path,
      });

      return {
        contents: result.css,
        loader: "css",
        resolveDir: path.dirname(args.path),
        watchFiles: [args.path],
      };
    });
  },
};

const options: esbuild.BuildOptions = {
  absWorkingDir: rootDir,
  entryPoints: ["src/index.ts"],
  outdir: outputDir,
  bundle: true,
  format: "esm",
  platform: "browser",
  sourcemap: isWatchMode,
  minify: !isWatchMode,
  plugins: [
    postcssPlugin,
    watchLogger,
  ],
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

build().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
