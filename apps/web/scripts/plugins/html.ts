import type { Plugin } from "esbuild";
import path from "node:path";
import { readFile, writeFile, mkdir } from "node:fs/promises";

interface HtmlPluginOptions {
  template: string;
  filename?: string;
  onGenerated?: () => void;
  liveReload?: boolean;
}

export function htmlPlugin({
  template,
  filename = "index.html",
  onGenerated,
  liveReload = false,
}: HtmlPluginOptions): Plugin {
  return {
    name: "html",

    setup(build) {
      build.onEnd(async (result) => {
        if (result.errors.length > 0 || !result.metafile) {
          return;
        }

        const outputDir = build.initialOptions.outdir;

        if (!outputDir) {
          throw new Error("esbuild outdir is required.");
        }

        const templatePath = path.resolve(
          build.initialOptions.absWorkingDir ?? process.cwd(),
          template
        );

        let html = await readFile(templatePath, "utf8");

        if (!html.includes("</head>") || !html.includes("</body>")) {
          throw new Error(
            "HTML template must contain </head> and </body>."
          );
        }

        const outputs = Object.entries(result.metafile.outputs);

        const jsFiles = outputs
          .filter(([file, info]) => file.endsWith(".js") && info.entryPoint)
          .map(([file]) => path.basename(file));

        const cssFiles = outputs
          .filter(([file]) => file.endsWith(".css"))
          .map(([file]) => path.basename(file));

        const stylesheets = cssFiles
          .map((file) => `<link rel="stylesheet" href="./${file}" />`)
          .join("\n    ");

        const scripts = jsFiles
          .map((file) => `<script type="module" src="./${file}"></script>`)
          .join("\n    ");

        const liveReloadScript = liveReload
          ? `
            <script>
              const protocol = location.protocol === "https:" ? "wss:" : "ws:";
              const socket = new WebSocket(\`\${protocol}//\${location.host}\`);

              socket.addEventListener("message", (event) => {
                if (event.data === "reload") {
                  location.reload();
                }
              });
            </script>
          `
          : "";

        html = html.replace(
          "</head>",
          `    ${stylesheets}\n  </head>`
        );

        html = html.replace(
          "</body>",
          `    ${scripts}\n    ${liveReloadScript}\n  </body>`
        );

        await mkdir(outputDir, { recursive: true });

        await writeFile(
          path.join(outputDir, filename),
          html,
          "utf8"
        );

        onGenerated?.();
      });
    },
  };
}
