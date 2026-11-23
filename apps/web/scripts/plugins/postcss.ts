import type { Plugin } from "esbuild";
import { readFile } from "node:fs/promises";
import path from "node:path";
import postcss from "postcss";
import autoprefixer from "autoprefixer";
import postcssNesting from "postcss-nesting";

const processor = postcss([
  postcssNesting(),
  autoprefixer(),
]);

export const postcssPlugin: Plugin = {
  name: "postcss",

  setup(build) {
    build.onLoad({ filter: /\.css$/ }, async (args) => {
      const source = await readFile(args.path, "utf8");

      const result = await processor.process(source, {
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
