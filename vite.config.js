import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import https from "node:https";

function mteamProxyPlugin() {
  return {
    name: "mteam-dev-proxy",
    configureServer(server) {
      server.middlewares.use("/mteam-api", (req, res) => {
        const requestPath = req.url || "/";
        const targetPath = requestPath.startsWith("/api") ? requestPath : `/api${requestPath}`;
        const chunks = [];

        req.on("data", (chunk) => chunks.push(chunk));
        req.on("end", () => {
          const body = Buffer.concat(chunks);

          const proxyReq = https.request(
            {
              hostname: "api.m-team.cc",
              port: 443,
              path: targetPath,
              method: req.method,
              headers: {
                "content-type": req.headers["content-type"] || "application/json",
                "content-length": body.length,
                "x-api-key": req.headers["x-api-key"] || "",
                "user-agent": "Mozilla/5.0"
              }
            },
            (proxyRes) => {
              res.statusCode = proxyRes.statusCode || 500;

              for (const [key, value] of Object.entries(proxyRes.headers)) {
                if (value !== undefined) {
                  res.setHeader(key, value);
                }
              }

              proxyRes.pipe(res);
            }
          );

          proxyReq.on("error", (error) => {
            res.statusCode = 502;
            res.setHeader("content-type", "application/json; charset=utf-8");
            res.end(
              JSON.stringify({
                code: "DEV_PROXY_ERROR",
                message: error.message
              })
            );
          });

          if (body.length > 0) {
            proxyReq.write(body);
          }

          proxyReq.end();
        });
      });
    }
  };
}

export default defineConfig({
  plugins: [vue(), mteamProxyPlugin()],
  base: "./",
  server: {
    host: "127.0.0.1",
    port: 5174
  }
});
