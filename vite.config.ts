import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    react(),
    // Plugin to serve Docusaurus docs from dist/docs
    {
      name: 'serve-docusaurus-docs',
      configureServer(server) {
        server.middlewares.use('/docs', (req, res, next) => {
          const docsPath = path.resolve(__dirname, 'dist/docs', req.url || '/');
          // Check if file exists
          if (fs.existsSync(docsPath)) {
            const stat = fs.statSync(docsPath);
            if (stat.isFile()) {
              // Serve the file
              res.setHeader('Content-Type', getContentType(docsPath));
              res.end(fs.readFileSync(docsPath));
              return;
            } else if (stat.isDirectory()) {
              // Try index.html in directory
              const indexPath = path.join(docsPath, 'index.html');
              if (fs.existsSync(indexPath)) {
                res.setHeader('Content-Type', 'text/html');
                res.end(fs.readFileSync(indexPath));
                return;
              }
            }
          }
          // If not found, try with /index.html
          const indexPath = path.resolve(__dirname, 'dist/docs', req.url || '/', 'index.html');
          if (fs.existsSync(indexPath)) {
            res.setHeader('Content-Type', 'text/html');
            res.end(fs.readFileSync(indexPath));
            return;
          }
          next();
        });
      },
    },
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/docs/**"],
    },
  },
  // Serve Docusaurus build from dist/docs in both dev and production
  publicDir: 'public',
}));

// Helper to determine content type
function getContentType(filePath: string): string {
  const ext = path.extname(filePath).toLowerCase();
  const contentTypes: Record<string, string> = {
    '.html': 'text/html',
    '.css': 'text/css',
    '.js': 'application/javascript',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon',
    '.woff': 'font/woff',
    '.woff2': 'font/woff2',
    '.ttf': 'font/ttf',
    '.eot': 'application/vnd.ms-fontobject',
  };
  return contentTypes[ext] || 'application/octet-stream';
}
