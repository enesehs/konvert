import type { NextConfig } from "next";

const isExport = process.env.OUTPUT_EXPORT === "true";

const nextConfig: NextConfig = {
  output: isExport ? "export" : "standalone",
  images: {
    unoptimized: true,
  },
  reactStrictMode: true,
  allowedDevOrigins: ["localhost", "127.0.0.1", "192.168.128.12", "192.168.*"],
};

export default nextConfig;
