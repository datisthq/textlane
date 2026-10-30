import { mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { afterAll, beforeAll, describe, expect, it } from "vite-plus/test"
import { buildAddon } from "../-test/addon.ts"
import * as html from "../generated/html.ts"
import * as md from "../generated/md.ts"
import * as nl from "../generated/nl.ts"
import { ADDON_ENV } from "../settings.ts"

let entry: typeof import("../index.ts")
let addonDir = ""

beforeAll(async () => {
  addonDir = mkdtempSync(join(tmpdir(), "textlane-addon-"))
  process.env[ADDON_ENV] = buildAddon(addonDir)
  entry = await import("../index.ts")
}, 600_000)

afterAll(() => {
  if (addonDir) rmSync(addonDir, { recursive: true, force: true })
})

const toHtml = () =>
  entry.textlane().use(md.parse()).use(md.gfm()).use(md.toHtml()).use(html.stringify())

describe("Processor", () => {
  it("processes Markdown to HTML", async () => {
    const file = await toHtml().process("# hi")
    expect(file).toEqual({ value: "<h1>hi</h1>", messages: [], data: {} })
  })

  it("processes synchronously", () => {
    expect(toHtml().processSync("~a~").value).toBe("<p><del>a</del></p>")
  })

  it("takes a list, tuples and names", () => {
    const processor = entry
      .textlane()
      .use([md.parse(), [md.toHtml(), { allowDangerousHtml: true }], "html.stringify"])
      .use(html.stringify({ allowDangerousHtml: true }))
    expect(processor.processSync("<b>a</b>").value).toBe("<p><b>a</b></p>")
  })

  it("takes a preset, its settings becoming data", () => {
    const processor = entry.textlane().use({
      plugins: [md.parse(), md.stringify()],
      settings: { bullet: "+" },
    })
    expect(processor.processSync("- a").value).toBe("* a\n")
    expect(processor.data("settings")).toEqual({ bullet: "+" })
  })

  it("reconfigures a plugin used again, merging object options", () => {
    const processor = entry
      .textlane()
      .use(md.parse())
      .use(md.stringify({ bullet: "+", emphasis: "_" }))
      .use(md.stringify({ bullet: "*" }))
    expect(processor.processSync("- *a*").value).toBe("* _a_\n")
  })

  it("ignores null and undefined", () => {
    expect(() => entry.textlane().use(null).use(undefined)).not.toThrow()
  })

  it("gets, sets and replaces data", () => {
    const processor = entry.textlane().data("a", 1)
    expect(processor.data("a")).toBe(1)
    expect(processor.data()).toEqual({ a: 1 })
    processor.data({ b: [2] })
    expect(processor.data()).toEqual({ b: [2] })
    expect(processor.data("missing")).toBeNull()
  })

  it("fails to change a frozen processor", () => {
    const processor = entry.textlane().use(md.parse()).freeze()
    expect(() => processor.use(md.gfm())).toThrow(
      expect.objectContaining({ code: "frozen-processor" }),
    )
    expect(() => processor.data("a", 1)).toThrow(
      expect.objectContaining({ code: "frozen-processor" }),
    )
  })

  it("parses, runs and stringifies", async () => {
    const processor = entry.textlane().use(md.parse()).use(md.stringify())
    const tree = processor.parse("*a*")
    expect(tree.type).toBe("root")
    expect(processor.runSync(tree)).toEqual(tree)
    expect(await processor.run(tree)).toEqual(tree)
    expect(processor.stringify(tree)).toBe("*a*\n")
  })

  it("reports messages", () => {
    const processor = entry
      .textlane()
      .use(nl.parse())
      .use(nl.repeatedWords())
      .use(nl.stringify())
    expect(processor.processSync("the the").messages).toEqual([
      {
        reason: "Unexpected repeated `the`, remove one occurrence",
        ruleId: "the",
        source: "retext-repeated-words",
        place: {
          start: { line: 1, column: 1, offset: 0 },
          end: { line: 1, column: 8, offset: 7 },
        },
      },
    ])
  })

  it("throws errors carrying the crate's code", async () => {
    expect(() => entry.textlane().use("md.missing")).toThrow(
      expect.objectContaining({ code: "unknown-plugin" }),
    )
    await expect(entry.textlane().process("a")).rejects.toMatchObject({
      code: "missing-parser",
      message: "Cannot `process` without `parser`",
    })
    expect(() =>
      entry.textlane().use(md.parse()).use("md.gfm", { singleTilde: "no" }).freeze(),
    ).toThrow(expect.objectContaining({ code: "invalid-options" }))
  })

  it("finishes asynchronous stages only in the async API", async () => {
    const processor = () =>
      entry.textlane().use("test.parseText").use("test.noop", { mode: "async" })
    const tree = { type: "root", children: [{ type: "text", value: "a" }] }
    expect(() => processor().runSync(tree)).toThrow(
      expect.objectContaining({ code: "async-in-sync" }),
    )
    expect(await processor().run(tree)).toEqual(tree)
  })

  it("lists the plugins the domain modules describe", async () => {
    const domains = await Promise.all(
      ["md", "html", "mdx", "nl"].map(name => import(`../generated/${name}.ts`)),
    )
    const described = domains.flatMap(domain =>
      Object.values(domain).map(describe =>
        typeof describe === "function" ? describe().name : undefined,
      ),
    )
    expect(described.toSorted()).toEqual(entry.pluginNames().toSorted())
  })

  it("reports the version", () => {
    expect(entry.version()).toMatch(/^\d+\.\d+\.\d+/)
  })
})
