import { DurableObject } from "cloudflare:workers";

interface Env {
  FORAGER_API: DurableObjectNamespace<ForagerApi>;
  FMP_API_KEY: string;
}

const PORT = 3001;
const INACTIVITY_TIMEOUT_MS = 30 * 60_000;

export class ForagerApi extends DurableObject<Env> {
  private starting: Promise<void> | undefined;

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    const container = ctx.container!;
    if (container.running) {
      void ctx.blockConcurrencyWhile(() =>
        container.setInactivityTimeout(INACTIVITY_TIMEOUT_MS),
      );
    }
  }

  async fetch(request: Request): Promise<Response> {
    this.starting ??= this.startAndWaitForPort().finally(() => {
      this.starting = undefined;
    });
    await this.starting;

    const url = new URL(request.url);
    url.protocol = "http:";
    url.host = "container";
    const forwarded = new Request(url, request);
    forwarded.headers.delete("host");
    return this.ctx.container!.getTcpPort(PORT).fetch(forwarded);
  }

  private async startAndWaitForPort(): Promise<void> {
    const container = this.ctx.container!;
  
    if (!container.running) {
      await container.start({
        image: container.images.api,
        instance: "lite",
        enableInternet: true,
        env: {
          BIND_ADDR: `0.0.0.0:${PORT}`,
          FMP_API_KEY: this.env.FMP_API_KEY,
        },
      });
    }
  
    await container.setInactivityTimeout(INACTIVITY_TIMEOUT_MS);
  
    const port = container.getTcpPort(PORT);
  
    for (let attempt = 0; attempt < 100; attempt++) {
      try {
        const res = await port.fetch("http://container/api/health", {
          signal: AbortSignal.timeout(1000),
        });
  
        await res.body?.cancel();
  
        if (res.ok) return;
      } catch {
        // Container may still be starting.
      }
  
      await scheduler.wait(200);
    }
  
    throw new Error(`Container did not become ready on port ${PORT}`);
  }
}

export default {
  fetch(request: Request, env: Env): Promise<Response> {
    return env.FORAGER_API.getByName("main").fetch(request);
  },
} satisfies ExportedHandler<Env>;