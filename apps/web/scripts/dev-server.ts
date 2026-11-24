import { createServer, request } from "node:http";
import type { IncomingMessage, ServerResponse } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { WebSocketServer, WebSocket } from "ws";

interface DevServerOptions {
  outputDir: string;
  host?: string;
  port?: number;
  apiPort?: number;
}

const mimeTypes: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
  ".map": "application/json",
};

function proxyToApi(
  req: IncomingMessage,
  res: ServerResponse,
  host: string,
  apiPort: number
) {
  const upstream = request(
    {
      hostname: host,
      port: apiPort,
      path: req.url,
      method: req.method,
      headers: req.headers,
    },
    (up) => {
      res.writeHead(up.statusCode ?? 502, up.headers);
      up.pipe(res);
    }
  );

  upstream.on("error", () => {
    if (!res.headersSent) {
      res.writeHead(502, { "Content-Type": "text/plain" });
    }
    res.end("API server unavailable (is cargo running?)");
  });

  res.on("close", () => upstream.destroy());
  req.pipe(upstream);
}

export function startDevServer({
  outputDir,
  host = "127.0.0.1",
  port = 3000,
  apiPort = 3001,
}: DevServerOptions) {
  const server = createServer(async (req, res) => {
    try {
      const pathname = new URL(req.url ?? "/", `http://${host}:${port}`).pathname;

      // 1. API calls go to Rust.
      if (pathname === "/api" || pathname.startsWith("/api/")) {
        proxyToApi(req, res, host, apiPort);
        return;
      }

      // 2. Static files, with traversal protection.
      const relativePath = decodeURIComponent(
        pathname === "/" ? "index.html" : pathname.slice(1)
      );
      const filePath = path.resolve(outputDir, relativePath);

      if (!filePath.startsWith(outputDir + path.sep)) {
        res.writeHead(403).end("Forbidden");
        return;
      }

      let target = filePath;
      let contents: Buffer;

      try {
        contents = await readFile(target);
      } catch {
        // 3. SPA fallback, but only for extension-less paths (routes).
        // A missing /index-abc.js should 404, not return HTML.
        if (path.extname(filePath) !== "") {
          res.writeHead(404).end("Not found");
          return;
        }
        target = path.join(outputDir, "index.html");
        contents = await readFile(target);
      }

      res.writeHead(200, {
        "Content-Type":
          mimeTypes[path.extname(target)] ?? "application/octet-stream",
        "Cache-Control": "no-store", // dev: never serve a stale bundle
      });
      res.end(contents);
    } catch {
      res.writeHead(404).end("Not found");
    }
  });

  const wss = new WebSocketServer({ server, path: "/__reload" });

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