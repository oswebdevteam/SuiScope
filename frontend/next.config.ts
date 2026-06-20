import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Emit a fully static site into `out/` so the Rust dashboard server
  // (crates/dashboard) can serve it via ServeDir — no Node runtime needed.
  output: "export",
  // Export each route as <route>/index.html so tower-http's ServeDir resolves
  // `/dashboard` to `dashboard/index.html`.
  trailingSlash: true,
  // next/image optimization needs a server; static export requires this off.
  images: { unoptimized: true },
};

export default nextConfig;
