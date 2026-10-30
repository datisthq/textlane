import { defineConfig } from "vite-plus"

const ignorePatterns = ["**/generated/**", "rust/target/**", "rust/vendor/**"]

export default defineConfig({
  fmt: {
    semi: false,
    printWidth: 90,
    arrowParens: "avoid",
    ignorePatterns: [...ignorePatterns, "**/CHANGELOG.md"],
    // Docs code snippets may run to 100 columns, the width the docs guard allows
    overrides: [{ files: ["**/*.md"], options: { printWidth: 100 } }],
  },
  lint: {
    ignorePatterns,
    options: {
      typeAware: false,
      typeCheck: false,
    },
  },
  test: {
    include: [".scripts/-test/*.unit.ts"],
    env: { NODE_OPTIONS: "--no-warnings" },
    testTimeout: 60 * 1000,
    silent: "passed-only",
  },
})
