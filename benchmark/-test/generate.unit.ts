import { mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { afterEach, describe, expect, it } from "vite-plus/test"
import { createDocument, generateDataset } from "../generate/generate.ts"
import { formatBytes, parseSize } from "../generate/size.ts"
import { proseProfile } from "../profiles/prose.ts"
import { readSnippets, specProfile } from "../profiles/spec.ts"

let directory = ""
afterEach(async () => {
  if (directory) await rm(directory, { recursive: true, force: true })
})

describe("generateDataset", () => {
  it("writes the same bytes for the same seed, and at least the size asked", async () => {
    directory = await mkdtemp(join(tmpdir(), "textlane-benchmark-test-"))
    const dataset = await generateDataset({
      profile: proseProfile,
      size: "4KB",
      directory,
    })
    const text = await readFile(dataset.path, "utf8")
    expect(dataset.bytes).toBeGreaterThanOrEqual(4096)
    expect(createDocument({ profile: proseProfile, bytes: 4096, seed: 1 })).toBe(text)
    expect(createDocument({ profile: proseProfile, bytes: 4096, seed: 2 })).not.toBe(text)
  })

  it("separates spec snippets with a thematic break at column 0", () => {
    const text = createDocument({ profile: specProfile, bytes: 2048, seed: 1 })
    expect(text).toMatch(/\n\n---\n\n$/)
  })
})

describe("readSnippets", () => {
  it("keeps only snippets that close every fence they open", () => {
    const snippets = readSnippets()
    expect(snippets.length).toBeGreaterThan(500)
    for (const snippet of snippets) {
      expect((snippet.match(/^ {0,3}(`{3,}|~{3,})/gm)?.length ?? 0) % 2).toBe(0)
    }
  })
})

describe("parseSize", () => {
  it("reads byte targets", () => {
    expect(parseSize("10KB")).toBe(10240)
    expect(parseSize("1.5MB")).toBe(1572864)
    expect(() => parseSize("10")).toThrow(/Invalid size/)
    expect(formatBytes(1572864)).toBe("1.5MB")
  })
})
