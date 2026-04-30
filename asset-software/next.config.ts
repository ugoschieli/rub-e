/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export", // CRITICAL: This must be 'export'
  images: {
    unoptimized: true, // Required for static exports
  },
};

export default nextConfig;
