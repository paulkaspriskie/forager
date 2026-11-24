import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { WebSocketServer, WebSocket } from "ws";

interface DevServerOptions {
  outputDir: string;
  host?: string;
  port?: number;
}

const mimeTypes: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
  ".map": "application/json",
};

export function startDevServer({
  outputDir,
  host = "127.0.0.1",
  port = 3000,
}: DevServerOptions) {
  const server = createServer(async (req, res) => {
    try {
      const pathname = new URL(
        req.url ?? "/",
        `http://${host}:${port}`
      ).pathname;

      const relativePath = decodeURIComponent(
        pathname === "/" ? "index.html" : pathname.slice(1)
      );

      const filePath = path.resolve(outputDir, relativePath);

      if (!filePath.startsWith(outputDir + path.sep)) {
        res.writeHead(403).end("Forbidden");
        return;
      }

      const contents = await readFile(filePath);
      const extension = path.extname(filePath);

      res.writeHead(200, {
        "Content-Type": mimeTypes[extension] ?? "application/octet-stream",
      });

      res.end(contents);
    } catch {
      res.writeHead(404).end("Not found");
    }
  });

  const wss = new WebSocketServer({ server });

  server.listen(port, host, () => {
    console.log(`Development server running at http://${host}:${port}`);
  });

  return {
    reload() {
      for (const client of wss.clients) {
        if (client.readyState === WebSocket.OPEN) {
          client.send("reload");
        }
      }
    },
  };
}
