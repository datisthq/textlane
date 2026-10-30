import { readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import rehypeShiki from "@shikijs/rehype"
import type { RehypeShikiOptions } from "@shikijs/rehype"
import {
  transformerMetaHighlight,
  transformerMetaWordHighlight,
  transformerNotationDiff,
  transformerNotationErrorLevel,
  transformerNotationFocus,
  transformerNotationHighlight,
  transformerRenderLineNumber,
} from "@shikijs/transformers"
import rehypeParse from "rehype-parse"
import rehypeStringify from "rehype-stringify"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { bundledLanguages } from "shiki"
import type { ShikiTransformer } from "shiki"
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

// `html.highlight`'s native options, each with the @shikijs/rehype options it stands for
type NativeOptions = Record<string, unknown> & {
  transformers?: (string | { name: string; [option: string]: unknown })[]
  metaProperty?: string
}

const TRANSFORMERS: Record<string, (options: never) => ShikiTransformer> = {
  notationDiff: transformerNotationDiff,
  notationFocus: transformerNotationFocus,
  notationHighlight: transformerNotationHighlight,
  notationErrorLevel: transformerNotationErrorLevel,
  metaHighlight: transformerMetaHighlight,
  metaWordHighlight: transformerMetaWordHighlight,
  renderLineNumber: transformerRenderLineNumber,
}

async function importTransformers() {
  const shiki = {
    repository: SHIKI.url,
    version: SHIKI.version,
    path: "packages/transformers",
    license: "MIT",
  }
  const fence = (language: string, code: string, meta = "") =>
    ["```" + language + (meta ? ` ${meta}` : ""), code, "```"].join("\n")
  const groups = []
  for (const group of TRANSFORMER_GROUPS) {
    const tests = []
    for (const test of group.tests) {
      const markdown =
        "markdown" in test ? test.markdown : fence(test.language, test.code, test.meta)
      tests.push(await createMarkdownTest(test.description, markdown, group.options))
    }
    groups.push({
      description: group.description,
      source: shiki,
      pipeline: [
        ["md.parse"],
        ["md.toHtml"],
        ["html.highlight", { themes: THEMES, ...group.options }],
        ["html.stringify"],
      ],
      tests,
    })
  }
  writeGroups({
    path: join(OUTPUT, "highlight-transformers.json"),
    repository: SHIKI.url,
    groups,
  })
}

async function createMarkdownTest(
  description: string,
  markdown: string,
  native: NativeOptions,
) {
  const { transformers = [], metaProperty, ...rest } = native
  const options: RehypeShikiOptions = {
    themes: THEMES,
    ...rest,
    transformers: transformers.map(entry => {
      const { name, ...options } = typeof entry === "string" ? { name: entry } : entry
      const create = TRANSFORMERS[name]
      if (!create) throw new Error(`no transformer \`${name}\``)
      return create(options as never)
    }),
    ...(metaProperty
      ? { parseMetaString: (meta: string) => ({ [metaProperty]: meta }) }
      : {}),
  }
  const processor = unified()
    .use(remarkParse)
    .use(remarkRehype)
    .use(rehypeShiki, options)
    .use(rehypeStringify)
  const html = String(await processor.process(markdown))
  written += 1
  return { description, input: markdown, expected: { html } }
}

type TransformerTest = { description: string } & (
  | { language: string; code: string; meta?: string }
  | { markdown: string }
)

const TRANSFORMER_GROUPS: {
  description: string
  options: NativeOptions
  tests: TransformerTest[]
}[] = [
  {
    description: "colorReplacements for every theme",
    options: { colorReplacements: { "#40a02b": "#2a7e16", "#eff1f5": "#ffffff" } },
    tests: [
      { description: "a string and the background", language: "js", code: 'let a = "b"' },
    ],
  },
  {
    description: "colorReplacements for one theme",
    options: { colorReplacements: { "catppuccin-mocha": { "#a6e3a1": "#00ff00" } } },
    tests: [
      { description: "the dark theme's string", language: "js", code: 'let a = "b"' },
    ],
  },
  {
    description: "addLanguageClass",
    options: { addLanguageClass: true },
    tests: [{ description: "a block", language: "js", code: "let a = 1" }],
  },
  {
    description: "the meta as a property of <pre>",
    options: { metaProperty: "data-meta" },
    tests: [
      {
        description: "a meta",
        language: "js",
        code: "let a = 1",
        meta: 'title="a.js" {1}',
      },
      { description: "no meta", language: "js", code: "let a = 1" },
    ],
  },
  {
    description: "inline code naming its language",
    options: { inline: "tailing-curly-colon" },
    tests: [
      { description: "a language", markdown: "Call `a(1){:js}` now" },
      { description: "no language", markdown: "Call `a(1)` now" },
      { description: "an unknown language", markdown: "Call `a(1){:nope}` now" },
    ],
  },
  {
    description: "inline code taking the fallback language",
    options: { inline: "tailing-curly-colon", fallbackLanguage: "text" },
    tests: [{ description: "an unknown language", markdown: "Call `a(1){:nope}` now" }],
  },
  {
    description: "transformerNotationDiff",
    options: { transformers: ["notationDiff"] },
    tests: [
      {
        description: "added and removed lines",
        language: "ts",
        code: "let a = 1 // [!code ++]\nlet b = 2 // [!code --]\nlet c = 3",
      },
      {
        description: "a notation on its own line marks the lines after it",
        language: "ts",
        code: "// [!code ++:2]\nlet a = 1\nlet b = 2\nlet c = 3",
      },
      { description: "a Python comment", language: "py", code: "a = 1 # [!code ++]" },
      {
        description: "an HTML comment",
        language: "html",
        code: "<a></a> <!-- [!code --] -->",
      },
      {
        description: "a block comment",
        language: "css",
        code: "a { color: red } /* [!code ++] */",
      },
      {
        description: "a JSX comment",
        language: "tsx",
        code: "const a = (\n  <div>\n    {/* [!code ++] */}\n    <b />\n  </div>\n)",
      },
      {
        description: "a comment that keeps more text",
        language: "ts",
        code: "let a = 1 // keep [!code ++]",
      },
    ],
  },
  {
    description: "transformerNotationDiff with its classes",
    options: {
      transformers: [
        {
          name: "notationDiff",
          classLineAdd: "plus",
          classLineRemove: ["minus"],
          classActiveCode: "x",
        },
      ],
    },
    tests: [
      {
        description: "both",
        language: "ts",
        code: "let a = 1 // [!code ++]\nlet b = 2 // [!code --]",
      },
    ],
  },
  {
    description: "transformerNotationFocus",
    options: { transformers: ["notationFocus"] },
    tests: [
      {
        description: "a focused line",
        language: "ts",
        code: "let a = 1\nlet b = 2 // [!code focus]",
      },
    ],
  },
  {
    description: "transformerNotationHighlight",
    options: { transformers: ["notationHighlight"] },
    tests: [
      {
        description: "highlight",
        language: "ts",
        code: "let a = 1 // [!code highlight]",
      },
      { description: "hl", language: "ts", code: "let a = 1 // [!code hl]" },
      {
        description: "uppercase",
        language: "ts",
        code: "let a = 1 // [!code HIGHLIGHT]",
      },
    ],
  },
  {
    description: "transformerNotationErrorLevel",
    options: { transformers: ["notationErrorLevel"] },
    tests: [
      {
        description: "each level",
        language: "ts",
        code: "a() // [!code error]\nb() // [!code warning]\nc() // [!code info]",
      },
    ],
  },
  {
    description: "transformerMetaHighlight",
    options: { transformers: ["metaHighlight"] },
    tests: [
      {
        description: "lines and a range",
        language: "ts",
        code: "a\nb\nc\nd",
        meta: "{1,3-4}",
      },
      { description: "no meta", language: "ts", code: "a\nb" },
    ],
  },
  {
    description: "transformerMetaHighlight zero-indexed",
    options: {
      transformers: [{ name: "metaHighlight", zeroIndexed: true, className: "hi" }],
    },
    tests: [{ description: "the first line", language: "ts", code: "a\nb", meta: "{0}" }],
  },
  {
    description: "transformerMetaWordHighlight",
    options: { transformers: ["metaWordHighlight"] },
    tests: [
      {
        description: "every occurrence",
        language: "ts",
        code: "const config = 1\nuse(config, config)",
        meta: "/config/",
      },
      {
        description: "a word across tokens",
        language: "ts",
        code: "let a = b.c(d)",
        meta: "/= b.c/",
      },
      {
        description: "a word that is a whole token",
        language: "ts",
        code: "let a = 1",
        meta: "/let/",
      },
      {
        description: "a word that is a whole line",
        language: "text",
        code: "abc\nabc",
        meta: "/abc/",
      },
      {
        description: "inside one token",
        language: "text",
        code: "hello world",
        meta: "/lo wo/",
      },
      {
        description: "two words",
        language: "ts",
        code: "let a = b + c",
        meta: "/a/ /c/",
      },
      {
        description: "an escaped slash",
        language: "text",
        code: "a/b and a/b",
        meta: "/a\\/b/",
      },
      {
        description: "nested words",
        language: "text",
        code: "foobar",
        meta: "/foobar/ /foo/",
      },
      {
        description: "a word that is not there",
        language: "ts",
        code: "let a",
        meta: "/zzz/",
      },
      { description: "no meta", language: "ts", code: "let a" },
    ],
  },
  {
    description: "transformerMetaWordHighlight with its class",
    options: { transformers: [{ name: "metaWordHighlight", className: "word" }] },
    tests: [
      { description: "a word", language: "ts", code: "let config", meta: "/config/" },
    ],
  },
  {
    description: "transformerMetaWordHighlight with notations",
    options: { transformers: ["metaWordHighlight", "notationHighlight"] },
    tests: [
      {
        description: "a word on a highlighted line",
        language: "ts",
        code: "let config = 1 // [!code hl]\nlet b",
        meta: "/config/",
      },
    ],
  },
  {
    description: "transformerRenderLineNumber",
    options: { transformers: [{ name: "renderLineNumber", start: 5 }] },
    tests: [{ description: "from five", language: "ts", code: "a\nb" }],
  },
  {
    description: "ansi code in a fence",
    options: {},
    tests: [
      {
        description: "colors and a decoration",
        language: "ansi",
        code: "\u001b[31mred\u001b[0m \u001b[1;32mbold green\u001b[0m",
      },
    ],
  },
  {
    description: "transformers run together, in order",
    options: {
      transformers: [
        "renderLineNumber",
        "metaHighlight",
        "notationDiff",
        "notationFocus",
      ],
    },
    tests: [
      {
        description: "numbers, meta lines and notations",
        language: "ts",
        code: "// [!code ++]\nlet a = 1 // [!code focus]\nlet b = 2",
        meta: "{3}",
      },
    ],
  },
]

// Escape sequences Shiki colors `ansi` code from, as a terminal would show them
const ESC = "\u001b["
const ANSI_TESTS: { description: string; code: string }[] = [
  { description: "plain text", code: "hello world\nsecond line" },
  {
    description: "the 8 foreground colors",
    code: [30, 31, 32, 33, 34, 35, 36, 37].map(n => `${ESC}${n}mc${n}`).join(" "),
  },
  {
    description: "the 8 background colors",
    code: [40, 41, 42, 43, 44, 45, 46, 47].map(n => `${ESC}${n}mc${n}`).join(" "),
  },
  {
    description: "the 8 bright foreground colors",
    code: [90, 91, 92, 93, 94, 95, 96, 97].map(n => `${ESC}${n}mc${n}`).join(" "),
  },
  {
    description: "the 8 bright background colors",
    code: [100, 101, 102, 103, 104, 105, 106, 107].map(n => `${ESC}${n}mc${n}`).join(" "),
  },
  {
    description: "foreground and background together",
    code: `${ESC}31;42mred on green${ESC}0m plain`,
  },
  {
    description: "256 colors, foreground",
    code: [0, 9, 15, 16, 21, 100, 196, 231, 232, 244, 255]
      .map(n => `${ESC}38;5;${n}mc${n}`)
      .join(""),
  },
  {
    description: "256 colors, background",
    code: [0, 9, 16, 124, 231, 240, 255].map(n => `${ESC}48;5;${n}mc${n}`).join(""),
  },
  {
    description: "256 colors past the table and not a number",
    code: `${ESC}38;5;256mpast${ESC}38;5;xmword${ESC}38;5;0x10mhex${ESC}38;5; mspace`,
  },
  {
    description: "RGB colors",
    code: `${ESC}38;2;255;128;0morange${ESC}48;2;10;20;30mon navy${ESC}0m ${ESC}38;2;300;-5;7mclamped`,
  },
  {
    description: "an RGB color missing a channel",
    code: `${ESC}38;2;1;2mshort${ESC}38;2;1;x;3;31mnot a number`,
  },
  { description: "bold", code: `${ESC}1mbold${ESC}0m plain` },
  { description: "dim", code: `${ESC}2mdim${ESC}0m plain` },
  { description: "italic", code: `${ESC}3mitalic${ESC}0m plain` },
  { description: "underline", code: `${ESC}4munderline${ESC}0m plain` },
  { description: "strikethrough", code: `${ESC}9mstruck${ESC}0m plain` },
  { description: "reverse", code: `${ESC}7mreversed${ESC}0m plain` },
  {
    description: "reverse with colors",
    code: `${ESC}7;31mred${ESC}42mred on green${ESC}39mon green`,
  },
  {
    description: "every decoration at once",
    code: `${ESC}1;3;4;9;31mall${ESC}0m`,
  },
  {
    description: "dim colors",
    code: `${ESC}2;31mred${ESC}38;5;100mtable${ESC}38;2;1;2;3mrgb${ESC}91mbright`,
  },
  {
    description: "hidden and overline make no style",
    code: `${ESC}8mhidden${ESC}28m ${ESC}53mover${ESC}55mline`,
  },
  {
    description: "resetting each decoration",
    code: `${ESC}1;2;3;4;7;9;31;42mall${ESC}22mno bold${ESC}23mno italic${ESC}24mno underline${ESC}27mno reverse${ESC}29mno strike`,
  },
  {
    description: "resetting colors",
    code: `${ESC}31;42mboth${ESC}39mbackground${ESC}49mnone${ESC}31mred${ESC}0mnone`,
  },
  {
    description: "21 resets bold and an empty sequence nothing",
    code: `${ESC}1;31mbold${ESC}21mred${ESC}mstill red`,
  },
  {
    description: "a reset after a color in one sequence",
    code: `${ESC}31;0mred${ESC}0;32mgreen${ESC}1;0mplain`,
  },
  {
    description: "styles carry over lines",
    code: `${ESC}32mgreen\nstill green${ESC}0m\nplain\r\n${ESC}1mbold\r\nbold`,
  },
  {
    description: "whitespace between colored words",
    code: `${ESC}31mred${ESC}0m   ${ESC}32mgreen${ESC}0m  `,
  },
  {
    description: "whitespace before an underlined word",
    code: `  ${ESC}4munderlined${ESC}0m ${ESC}9m struck`,
  },
  {
    description: "a line of escapes alone",
    code: `${ESC}31m\n${ESC}0m${ESC}1m\ntext`,
  },
  { description: "an escape starting no sequence", code: `a\u001bb ${ESC}31mc` },
  { description: "a sequence never closed", code: `${ESC}31mred ${ESC}1;32 green` },
  { description: "unknown codes", code: `${ESC}5;6;10;50;99;108;-1;foomblink` },
  {
    description: "a test run's log",
    code: [
      `${ESC}1m${ESC}32m✓${ESC}39m${ESC}22m parses ${ESC}2m(3 ms)${ESC}22m`,
      `${ESC}1m${ESC}31m✗${ESC}39m${ESC}22m formats`,
      `  ${ESC}31mExpected:${ESC}39m ${ESC}32m"a"${ESC}39m`,
      `  ${ESC}90mat ${ESC}4mtest.ts:1:2${ESC}24m${ESC}39m`,
    ].join("\n"),
  },
  { description: "an empty block", code: "" },
]

async function importAnsi() {
  const shiki = {
    repository: SHIKI.url,
    version: SHIKI.version,
    path: "packages/core/src/highlight/code-to-tokens-ansi.ts",
    license: "MIT",
  }
  const settings: { description: string; options: RehypeShikiOptions }[] = [
    { description: `${THEMES.light} / ${THEMES.dark}`, options: { themes: THEMES } },
    {
      description: "github-light / github-dark",
      options: { themes: { light: "github-light", dark: "github-dark" } },
    },
    {
      description: "one theme with an alpha in its foreground",
      options: { theme: "vitesse-dark" },
    },
    { description: "one theme without terminal colors", options: { theme: "dark-plus" } },
    {
      description: "colorReplacements",
      options: {
        themes: THEMES,
        colorReplacements: {
          "#d20f39": "#ff0000",
          "#4c4f69": "#000000",
          "catppuccin-mocha": { "#a6e3a1": "#00ff00" },
        },
      },
    },
  ]
  const groups = []
  for (const setting of settings) {
    const tests = []
    for (const test of ANSI_TESTS)
      tests.push(
        await createTest({
          description: test.description,
          html: codeBlock(test.code, "ansi"),
          options: setting.options,
        }),
      )
    groups.push({
      description: `ansi under ${setting.description}`,
      source: shiki,
      pipeline: [
        ["html.parse", { fragment: true }],
        ["html.highlight", setting.options],
        ["html.stringify"],
      ],
      tests,
    })
  }
  writeGroups({
    path: join(OUTPUT, "highlight-ansi.json"),
    repository: SHIKI.url,
    groups,
  })
}

await importTransformers()
await importAnsi()
console.log(`highlight: ${written} cases`)
