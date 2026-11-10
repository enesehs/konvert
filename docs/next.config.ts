import type { NextConfig } from "next";

const isExport = process.env.OUTPUT_EXPORT === "true";
const basePath = process.env.BASE_PATH ?? (isExport ? "/konvert" : "");

const nextConfig: NextConfig = {
  output: isExport ? "export" : "standalone",
  basePath: basePath || undefined,
  assetPrefix: basePath || undefined,
  env: {
    NEXT_PUBLIC_BASE_PATH: basePath,
  },
  images: {
    unoptimized: true,
  },
  reactStrictMode: true,
  allowedDevOrigins: ["localhost", "127.0.0.1", "192.168.128.12", "192.168.*"],
};

export default nextConfig;
