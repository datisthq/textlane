import { readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import rehypeShiki from "@shikijs/rehype"
import type { RehypeShikiOptions } from "@shikijs/rehype"
import rehypeParse from "rehype-parse"
import rehypeStringify from "rehype-stringify"
import { bundledLanguages } from "shiki"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html")
// The grammar samples giallo tests against, at the commit its submodule pins
const SAMPLES = {
  url: "https://github.com/shikijs/textmate-grammars-themes",
  commit: "4cb1625ad87a07fb4674fe97b921a8f756f4694b",
}
const SHIKI = { url: "https://github.com/shikijs/shiki", version: "4.5.0" }
// livemark's default theme pair; a sample's first lines exercise its grammar at a tenth the size
const THEMES = { light: "catppuccin-latte", dark: "catppuccin-mocha" }
const MAX_LINES = 30
const THEME_MATRIX = [
  { light: "github-light", dark: "github-dark" },
  { light: "min-light", dark: "min-dark" },
  { light: "vitesse-light", dark: "vitesse-dark" },
  { light: "one-light", dark: "one-dark-pro" },
]
const MATRIX_LANGUAGES = [
  "javascript",
  "typescript",
  "python",
  "rust",
  "css",
  "html",
  "json",
  "markdown",
  "shellscript",
  "yaml",
]

let written = 0
await importHighlight()
console.log(`highlight: ${written} cases`)

async function importHighlight() {
  const root = join(
    fetchSource({
      repository: SAMPLES.url,
      commit: SAMPLES.commit,
      paths: ["/samples/"],
    }),
    "samples",
  )
  const samples = new Map<string, string>()
  for (const file of readdirSync(root)
    .filter(name => name.endsWith(".sample"))
    .sort()) {
    const language = file.replace(/\.sample$/, "")
    if (!(language in bundledLanguages)) continue
    samples.set(
      language,
      readFileSync(join(root, file), "utf8").split("\n").slice(0, MAX_LINES).join("\n"),
    )
  }
  const options = { themes: THEMES }
  const sampleTests = []
  for (const [language, code] of samples)
    sampleTests.push(
      await createTest({
        description: language,
        html: codeBlock(code, language),
        options,
      }),
    )
  const matrix = []
  for (const themes of THEME_MATRIX) {
    const tests = []
    for (const language of MATRIX_LANGUAGES) {
      const code = samples.get(language)
      if (code)
        tests.push(
          await createTest({
            description: language,
            html: codeBlock(code, language),
            options: { themes },
          }),
        )
    }
    matrix.push({ themes, tests })
  }
  const fallback = { themes: THEMES, defaultLanguage: "text", fallbackLanguage: "text" }
  const behaviour = [
    await createTest({
      description: "a block without a language is left alone",
      html: "<pre><code>let a = 1</code></pre>",
      options,
    }),
    await createTest({
      description: "a block in an unknown language is left alone",
      html: codeBlock("let a = 1", "not-a-language"),
      options,
    }),
    await createTest({
      description: "a block without a language takes the default language",
      html: "<pre><code>let a = 1</code></pre>",
      options: fallback,
    }),
    await createTest({
      description: "a block in an unknown language takes the fallback language",
      html: codeBlock("let a = 1", "not-a-language"),
      options: fallback,
    }),
    await createTest({
      description: "inline code is left alone",
      html: '<p>Use <code class="language-js">a()</code> here.</p>',
      options,
    }),
    await createTest({
      description: "an empty block",
      html: codeBlock("", "javascript"),
      options,
    }),
    await createTest({
      description: "entities in code are decoded before highlighting",
      html: codeBlock("a < b && c > d", "javascript"),
      options,
    }),
  ]
  const source = (path: string) => ({
    repository: SAMPLES.url,
    commit: SAMPLES.commit,
    path,
    license: "MIT",
  })
  const shiki = {
    repository: SHIKI.url,
    version: SHIKI.version,
    path: "packages/rehype",
    license: "MIT",
  }
  const pipeline = (highlight: object) => [
    ["html.parse", { fragment: true }],
    ["html.highlight", highlight],
    ["html.stringify"],
  ]
  writeGroups({
    path: join(OUTPUT, "highlight.json"),
    repository: SAMPLES.url,
    groups: [
      {
        description: `every Shiki ${SHIKI.version} grammar's sample (first ${MAX_LINES} lines)`,
        comment:
          "Expected HTML is @shikijs/rehype's; giallo must match it with the grammar versions Shiki bundles",
        source: source("samples"),
        pipeline: pipeline(options),
        tests: sampleTests,
      },
      ...matrix.map(entry => ({
        description: `samples under ${entry.themes.light} / ${entry.themes.dark}`,
        source: source("samples"),
        pipeline: pipeline({ themes: entry.themes }),
        tests: entry.tests,
      })),
    ],
  })
  writeGroups({
    path: join(OUTPUT, "highlight.json"),
    repository: SHIKI.url,
    groups: [
      {
        description: "code blocks without a usable language",
        source: shiki,
        pipeline: pipeline(options),
        tests: behaviour.slice(0, 2).concat(behaviour.slice(4)),
      },
      {
        description: "default and fallback languages",
        source: shiki,
        pipeline: pipeline(fallback),
        tests: behaviour.slice(2, 4),
      },
    ],
  })
}

async function createTest(props: {
  description: string
  html: string
  options: RehypeShikiOptions
}) {
  const processor = unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeShiki, props.options)
    .use(rehypeStringify)
  const html = String(await processor.process(props.html))
  written += 1
  return { description: props.description, input: props.html, expected: { html } }
}

function codeBlock(code: string, language: string) {
  const escaped = code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
  return `<pre><code class="language-${language}">${escaped}</code></pre>`
}
