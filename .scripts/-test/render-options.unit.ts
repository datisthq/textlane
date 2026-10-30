import { describe, expect, it } from "vite-plus/test"
import { renderOptions, replaceSections } from "../render-options.ts"
import type { Schema } from "../render-options.ts"

const definitions: Record<string, Schema> = {
  GfmOptions: {
    type: "object",
    properties: {
      singleTilde: {
        type: "boolean",
        default: true,
        description: "Let one tilde | form strikethrough.",
      },
    },
  },
  FrontmatterOptions: {
    anyOf: [
      { type: "string", enum: ["yaml", "toml"] },
      { $ref: "#/$defs/Matter" },
      { type: "array", items: { $ref: "#/$defs/Matter" } },
    ],
  },
  Matter: {
    type: "object",
    properties: { type: { type: "string" }, anywhere: { type: "boolean" } },
    required: ["type"],
  },
  StringifyOptions: {
    type: "object",
    properties: {
      unsafe: {
        type: "array",
        items: { type: "string" },
      },
    },
  },
}

const gfm = {
  name: "md.gfm",
  upstream: "remark-gfm",
  options: { $ref: "#/$defs/GfmOptions" },
}

describe("renderOptions", () => {
  it("renders camelCase keys and TypeScript types", () => {
    const text = renderOptions({ language: "typescript", plugin: gfm, definitions })
    expect(text).toContain("| `singleTilde` | `boolean` | `true`  |")
    expect(text).toContain("[remark-gfm](https://www.npmjs.com/package/remark-gfm)")
    expect(text).toContain("Let one tilde \\| form strikethrough.")
  })

  it("renders camelCase keys and Python types", () => {
    const text = renderOptions({ language: "python", plugin: gfm, definitions })
    expect(text).toContain("| `singleTilde` | `bool` | `True`  |")
  })

  it("renders the JSON keys and types Rust's Pipeline takes", () => {
    const text = renderOptions({ language: "rust", plugin: gfm, definitions })
    expect(text).toContain("| `singleTilde` | `boolean` | `true`  |")
  })

  it("renders a union of refs readably in every language", () => {
    const plugin = {
      name: "md.frontmatter",
      upstream: "remark-frontmatter",
      options: { $ref: "#/$defs/FrontmatterOptions" },
    }
    for (const language of ["typescript", "python", "rust"] as const) {
      const text = renderOptions({ language, plugin, definitions })
      expect(text).not.toContain("[object Object]")
      expect(text).toContain("Matter")
      expect(text).toMatch(/\| `(type|anywhere)` \|/)
    }
  })

  it("says an empty options object is no options, without the plugin summary", () => {
    const plugin = {
      name: "md.parse",
      upstream: "remark-parse",
      options: { $ref: "#/$defs/ParseOptions" },
    }
    const text = renderOptions({
      language: "typescript",
      plugin,
      definitions: { ParseOptions: { type: "object", description: "`md.parse`: it." } },
    })
    expect(text).toContain("No options.")
    expect(text).not.toContain("`md.parse`: it.")
  })

  it("says a unit plugin takes no options", () => {
    const plugin = { name: "md.gemoji", upstream: "remark-gemoji", options: null }
    const text = renderOptions({ language: "typescript", plugin, definitions })
    expect(text).toContain("No options.")
  })
})

describe("renderOptions tables", () => {
  it("pads every column to its widest cell", () => {
    const text = renderOptions({ language: "typescript", plugin: gfm, definitions })
    const rows = text.split("\n").filter(line => line.startsWith("|"))
    expect(new Set(rows.map(row => row.length)).size).toBe(1)
    expect(rows[1]).toMatch(/^\| -+ \| -+ \| -+ \| -+ \|$/)
  })
})

describe("replaceSections", () => {
  const text = [
    "# Plugins",
    "",
    "Hand-written.",
    "<!-- generated:options md.gfm -->",
    "stale",
    "<!-- /generated -->",
    "Tail.",
    "",
  ].join("\n")

  it("replaces only between the markers, padded by blank lines as the formatter wants", () => {
    const result = replaceSections({ text, names: ["md.gfm"], render: () => "fresh" })
    expect(result.text).toBe(text.replace("stale", "\nfresh\n"))
    expect(result.missing).toEqual([])
  })

  it("is idempotent", () => {
    const render = () => "fresh"
    const once = replaceSections({ text, names: ["md.gfm"], render }).text
    expect(replaceSections({ text: once, names: ["md.gfm"], render }).text).toBe(once)
  })

  it("reports a plugin with no section", () => {
    const result = replaceSections({
      text,
      names: ["md.gfm", "md.gemoji"],
      render: () => "fresh",
    })
    expect(result.missing).toEqual(["md.gemoji"])
  })
})
