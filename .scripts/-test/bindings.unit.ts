import { spawnSync } from "node:child_process"
import { resolve } from "node:path"
import { describe, expect, it } from "vite-plus/test"

const SCRIPT = resolve(import.meta.dirname, "..", "generate-bindings.ts")

describe("the generated bindings", () => {
  it(
    "match the crate's plugins and option structs",
    () => {
      const result = spawnSync("node", [SCRIPT, "--check"], { encoding: "utf8" })
      expect(result.stderr).not.toContain("out of date")
      expect(result.status).toBe(0)
    },
    10 * 60 * 1000,
  )
})
