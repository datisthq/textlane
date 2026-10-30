import { describe, expect, it } from "vite-plus/test"
import * as html from "../generated/html.ts"
import * as md from "../generated/md.ts"
import * as mdx from "../generated/mdx.ts"
import * as nl from "../generated/nl.ts"

describe("domain modules", () => {
  it("describe a plugin by its name, options only when given", () => {
    expect(md.toHtml()).toEqual({ name: "md.toHtml" })
    expect(md.toHtml({ allowDangerousHtml: true })).toEqual({
      name: "md.toHtml",
      options: { allowDangerousHtml: true },
    })
    expect(md.frontmatter(["yaml", { type: "custom", fence: "+++" }])).toEqual({
      name: "md.frontmatter",
      options: ["yaml", { type: "custom", fence: "+++" }],
    })
  })

  it("describe plugins without options", () => {
    expect(md.gemoji()).toEqual({ name: "md.gemoji" })
    expect(html.toNaturalLanguage()).toEqual({ name: "html.toNaturalLanguage" })
    expect(nl.indefiniteArticle()).toEqual({ name: "nl.indefiniteArticle" })
  })

  it("name each function after its plugin", () => {
    const domains = { md, html, mdx, nl }
    for (const [domain, functions] of Object.entries(domains)) {
      for (const [name, describe] of Object.entries(functions)) {
        expect(describe().name).toBe(`${domain}.${name}`)
      }
    }
  })
})
