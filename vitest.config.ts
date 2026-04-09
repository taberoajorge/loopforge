import { defineConfig } from "vitest/config";
import baseConfig from "./vite.config";

export default defineConfig(async () => {
  const viteConfig = await baseConfig();
  return {
    ...viteConfig,
    test: {
      environment: "jsdom",
      setupFiles: ["src/test/setup.ts"],
      include: ["src/**/*.test.{ts,tsx}", "src/**/*.spec.{ts,tsx}"],
      passWithNoTests: true,
    },
  };
});
