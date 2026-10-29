import { defineConfig } from "vite-plus"

export default defineConfig({
  test: {
    include: ["**/*.unit.ts"],
    exclude: ["**/node_modules/**"],
    env: { NODE_OPTIONS: "--no-warnings" },
    silent: "passed-only",
  },
})
