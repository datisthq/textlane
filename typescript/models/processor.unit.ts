import { mkdtempSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import type { Root as HastRoot } from "hast"
import type { Root as MdastRoot } from "mdast"
import type { Root as NlcstRoot } from "nlcst"
import { afterAll, beforeAll, describe, expect, expectTypeOf, it } from "vite-plus/test"
import { buildAddon, removeAddon } from "../-test/addon.ts"
import * as html from "../generated/html.ts"
import * as md from "../generated/md.ts"
import * as nl from "../generated/nl.ts"
import { ADDON_ENV } from "../settings.ts"
import type { Node } from "./file.ts"
import type { HostFile, HostMessage } from "./vfile.ts"
import type { Processor } from "./processor.ts"

let entry: typeof import("../index.ts")
let addonDir = ""

beforeAll(async () => {
  addonDir = mkdtempSync(join(tmpdir(), "textlane-addon-"))
  process.env[ADDON_ENV] = buildAddon(addonDir)
  entry = await import("../index.ts")
}, 600_000)

afterAll(() => {
  if (addonDir) removeAddon(addonDir)
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

  it("moves trees past a million values and a hundred levels deep", () => {
    const keep = () => (tree: Node) => tree
    const processor = entry.textlane().use([md.parse(), keep, md.stringify()])
    for (const text of ["- *a*\n".repeat(30_000), "> ".repeat(200) + "a\n"]) {
      const tree = processor.parse(text)
      expect(processor.runSync(tree)).toEqual(tree)
      expect(processor.stringify(tree)).toBe(processor.processSync(text).value)
    }
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

  it("processes documents concurrently, each as processSync does", async () => {
    const processor = toHtml()
    const documents = Array.from({ length: 50 }, (_, index) => `# ${index}\n\n~a~ *b*`)
    const files = await Promise.all(
      documents.map(document => processor.process(document)),
    )
    expect(files.map(file => file.value)).toEqual(
      documents.map(document => processor.processSync(document).value),
    )
  })

  it("rejects with the crate's code when a document fails off the main thread", async () => {
    const processor = entry
      .textlane()
      .use(md.parse())
      .use(md.mdx())
      .use(md.toHtml())
      .use(html.stringify())
    await expect(processor.process("<a")).rejects.toMatchObject({
      code: "micromark-extension-mdx-jsx:unexpected-eof",
    })
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

  it("hands the file's text to a run, as a bridge into nlcst needs", () => {
    const processor = () => entry.textlane().use(md.parse()).use(md.toNaturalLanguage())
    const tree = processor().parse({ value: "a b" })
    const withText = processor().runSync(tree, "a b")
    expect(withText.position).toEqual(
      processor().runSync(tree, { value: "a b" }).position,
    )
    expect(withText).not.toEqual(processor().runSync(tree))
  })

  it("types the tree its parser parses, through later plugins", () => {
    const tree = entry.textlane().use(md.parse()).use(md.gfm()).parse("# a")
    expectTypeOf(tree).toEqualTypeOf<MdastRoot>()
    expect(tree.children[0]?.type).toBe("heading")
    expectTypeOf(entry.textlane().use(html.parse()).parse("")).toEqualTypeOf<HastRoot>()
    expectTypeOf(entry.textlane().use(nl.parse()).parse("")).toEqualTypeOf<NlcstRoot>()
    expectTypeOf(entry.textlane().parse).returns.toEqualTypeOf<Node>()
  })

  it("compiles a typed tree as given", () => {
    const root: MdastRoot = { type: "root", children: [{ type: "thematicBreak" }] }
    expect(entry.textlane().use(md.stringify()).stringify(root)).toBe("***\n")
  })

  it("reports the version", () => {
    expect(entry.version()).toMatch(/^\d+\.\d+\.\d+/)
  })

  describe("JavaScript plugins", () => {
    const strongEmphasis = () => (tree: Node) => {
      visit(tree, node => {
        if (node.type === "emphasis") node.type = "strong"
      })
    }

    it("runs a transformer between native stages", () => {
      const processor = entry
        .textlane()
        .use(md.parse())
        .use(strongEmphasis)
        .use(md.toHtml())
        .use(html.stringify())
      expect(processor.processSync("*a*").value).toBe("<p><strong>a</strong></p>")
    })

    it("hands the hast tree to a plugin after md.toHtml", () => {
      const addClass = () => (tree: Node) => {
        visit(tree, node => {
          if (node.tagName === "p") node.properties = { className: ["lead"] }
        })
      }
      const processor = entry
        .textlane()
        .use([md.parse(), md.toHtml(), addClass, html.stringify()])
      expect(processor.processSync("a").value).toBe('<p class="lead">a</p>')
    })

    it("calls the attacher with the processor and merged options", () => {
      const seen: unknown[] = []
      function record(this: Processor, options: unknown) {
        seen.push(options, this.data("key"))
      }
      const processor = entry
        .textlane()
        .data("key", "value")
        .use(md.parse())
        .use(record, { a: { b: 1 } })
        .use([[record, { a: { c: 2 } }]])
        .use(md.stringify())
      processor.processSync("a")
      expect(seen).toEqual([{ a: { b: 1, c: 2 } }, "value"])
    })

    it("replaces the tree with the one a transformer returns", async () => {
      const replace = () => async () => ({
        type: "root",
        children: [{ type: "paragraph", children: [{ type: "text", value: "b" }] }],
      })
      const processor = entry.textlane().use([md.parse(), replace, md.stringify()])
      expect((await processor.process("a")).value).toBe("b\n")
    })

    it("fails a synchronous run on a transformer that finishes later", () => {
      const later = () => async () => {}
      const processor = entry.textlane().use([md.parse(), later, md.stringify()])
      expect(() => processor.processSync("a")).toThrow(
        expect.objectContaining({ code: "async-in-sync" }),
      )
    })

    it("waits for a transformer's next callback", async () => {
      const callback =
        () => (_tree: Node, _file: unknown, next: (e?: unknown) => void) => {
          setTimeout(() => next(), 1)
          return undefined
        }
      const processor = entry.textlane().use([md.parse(), callback, md.stringify()])
      expect((await processor.process("a")).value).toBe("a\n")
    })

    it("keeps the messages and data a plugin reports", async () => {
      const warn = () => (tree: Node, file: HostFile) => {
        file.data.seen = true
        file.message("look", {
          place: { line: 1, column: 1 },
          ruleId: "rule",
          source: "js",
        })
        return tree
      }
      const file = await entry
        .textlane()
        .use([md.parse(), warn, md.stringify()])
        .process("a")
      expect(file.data).toEqual({ seen: true })
      expect(file.messages).toEqual([
        {
          reason: "look",
          ruleId: "rule",
          source: "js",
          place: {
            start: { line: 1, column: 1, offset: null },
            end: { line: 1, column: 1, offset: null },
          },
        },
      ])
    })

    it("skips a plugin turned off", () => {
      const fail = () => () => {
        throw new Error("ran")
      }
      const processor = entry
        .textlane()
        .use([md.parse(), fail, md.stringify()])
        .use(fail, false)
      expect(processor.processSync("a").value).toBe("a\n")
    })

    it("rethrows what a transformer throws", async () => {
      const fail = () => () => {
        throw new Error("boom")
      }
      const processor = entry.textlane().use([md.parse(), fail, md.stringify()])
      await expect(processor.process("a")).rejects.toThrow("boom")
    })

    it("fails the run on an Error a transformer returns", () => {
      const fail = () => () => new Error("returned")
      const processor = entry.textlane().use([md.parse(), fail, md.stringify()])
      expect(() => processor.processSync("a")).toThrow("returned")
    })

    it("gives plugins the file's path and keeps it in the result", () => {
      const seen: unknown[] = []
      const record = () => (_tree: Node, file: HostFile) => {
        seen.push(file.path, file.dirname, file.stem, file.extname, String(file))
      }
      const processor = entry.textlane().use([md.parse(), record, md.stringify()])
      const file = processor.processSync({ value: "a", path: "docs/intro.md" })
      expect(seen).toEqual(["docs/intro.md", "docs", "intro", ".md", "a"])
      expect(file.path).toBe("docs/intro.md")
    })

    it("reports messages as vfile does", () => {
      const reported: HostMessage[] = []
      const report = () => (_tree: Node, file: HostFile) => {
        reported.push(
          file.message("a", { line: 1, column: 2 }, "remark-lint:rule"),
          file.info(new Error("b")),
        )
        file.fail("c", { place: { start: point(1, 1), end: point(1, 3) }, ruleId: "x" })
      }
      const processor = entry.textlane().use([md.parse(), report, md.stringify()])
      expect(() => processor.processSync({ value: "abc", path: "a.md" })).toThrow(
        expect.objectContaining({
          reason: "c",
          fatal: true,
          name: "1:1-1:3",
          file: "a.md",
        }),
      )
      expect(reported[0]).toMatchObject({
        reason: "a",
        source: "remark-lint",
        ruleId: "rule",
        fatal: false,
        line: 1,
        column: 2,
        name: "1:2",
      })
      expect(reported[1]).toMatchObject({ reason: "b", fatal: null, name: "1:1" })
      expect(reported[1]?.cause).toBeInstanceOf(Error)
    })

    it("shows plugins what native stages reported", () => {
      const counts: number[] = []
      const count = () => (_tree: Node, file: HostFile) => {
        counts.push(file.messages.length)
        file.message("from js")
      }
      const processor = entry
        .textlane()
        .use([md.parse(), ["test.message", { reason: "native" }], count, md.stringify()])
      const file = processor.processSync("a")
      expect(counts).toEqual([1])
      expect(file.messages.map(message => message.reason)).toEqual(["native", "from js"])
    })

    it("merges the data native stages and plugins store in the file", () => {
      const mark = () => (_tree: Node, file: HostFile) => {
        file.data.seen = true
      }
      const file = entry
        .textlane()
        .use([md.parse(), md.toHtml(), html.extractToc(), mark, html.stringify()])
        .processSync("# A")
      expect(file.data).toEqual({ toc: [{ depth: 1, value: "A" }], seen: true })
    })

    it("runs a tree through native and JavaScript stages", async () => {
      const tree = { type: "root", children: [{ type: "emphasis", children: [] }] }
      const result = await entry.textlane().use(strongEmphasis).use(md.toHtml()).run(tree)
      expect(result).toEqual({
        type: "root",
        children: [{ type: "element", tagName: "strong", properties: {}, children: [] }],
      })
    })
  })
})

function visit(tree: Node, visitor: (node: Node) => void) {
  visitor(tree)
  const children = Array.isArray(tree.children) ? tree.children : []
  for (const child of children) visit(child, visitor)
}

function point(line: number, column: number) {
  return { line, column, offset: null }
}
