import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import rehypeKatex from "rehype-katex"
import type { Options as KatexOptions } from "rehype-katex"
import rehypeParse from "rehype-parse"
import rehypeStringify from "rehype-stringify"
import { unified } from "unified"
import { parse } from "yaml"
import { writeGroups } from "../actions/group/write.ts"
import { isStressInput } from "../actions/input/check.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html", "katex.json")
// The release katex-rs 0.3.0 tracks, and rehype-katex's own tests
const KATEX = {
  url: "https://github.com/KaTeX/KaTeX",
  commit: "7f46076ffd2b45d43829ffebe001595ab5f60597",
  version: "0.18.5",
}
const REHYPE_KATEX = {
  url: "https://github.com/remarkjs/remark-math",
  commit: "88a9497e1ede93b958237c85edbf5651faeca7af",
  version: "7.0.1",
}
const SORTED =
  "Expected HTML is rehype-katex's over KaTeX 0.18.5, with each element's attributes sorted: `class`, `style`, then the rest by name: katex-rs 0.3.0 keeps them in a randomly seeded hash map, and html.katex sorts them so its output is stable"

type Entry = string | { tex: string; display?: number; macros?: Record<string, string> }

let written = 0
await importScreenshots()
await importBehaviour()
console.log(`katex: ${written} cases`)

async function importScreenshots() {
  const root = fetchSource({
    repository: KATEX.url,
    commit: KATEX.commit,
    paths: ["/test/screenshotter/ss_data.yaml"],
  })
  const path = "test/screenshotter/ss_data.yaml"
  const data: Record<string, Entry> = parse(readFileSync(join(root, path), "utf8"))
  // Cases sharing their options share a group, as a group fixes one pipeline
  const groups = new Map<string, { options: KatexOptions; tests: object[] }>()
  for (const [name, entry] of Object.entries(data)) {
    const { tex, display, macros } = typeof entry === "string" ? { tex: entry } : entry
    const input = wrapMath(tex.replace(/\n$/, ""), Boolean(display))
    if (isStressInput(input)) continue
    const options: KatexOptions = macros ? { macros } : {}
    const key = JSON.stringify(options)
    const group = groups.get(key) ?? { options, tests: [] }
    group.tests.push(await createTest({ description: name, input, options }))
    groups.set(key, group)
  }
  writeGroups({
    path: OUTPUT,
    repository: KATEX.url,
    groups: [...groups.values()].map(group => ({
      description:
        Object.keys(group.options).length === 0
          ? `KaTeX ${KATEX.version}'s screenshot formulas`
          : `KaTeX ${KATEX.version}'s screenshot formulas with ${Object.keys(group.options).join(", ")}`,
      comment: SORTED,
      source: {
        repository: KATEX.url,
        version: KATEX.version,
        commit: KATEX.commit,
        path,
        license: "MIT",
      },
      pipeline: createPipeline(group.options),
      tests: group.tests,
    })),
  })
}

async function importBehaviour() {
  const html = async (description: string, input: string, options: KatexOptions = {}) =>
    createTest({ description, input, options })
  const groups: { description: string; options: KatexOptions; tests: object[] }[] = [
    {
      description: "math elements",
      options: {},
      tests: [
        await html(
          "inline math",
          '<p>a <code class="language-math math-inline">\\alpha</code> b</p>',
        ),
        await html(
          "display math",
          '<pre><code class="language-math math-display">\\sum_{i=0}^n i</code></pre>',
        ),
        await html(
          "a math fence replaces its `pre`",
          '<pre><code class="language-math">x^2</code></pre>',
        ),
        await html(
          "any element of class `math-display`",
          '<div class="math-display">\\frac{1}{2}</div>',
        ),
        await html(
          "any element of class `math-inline`",
          '<span class="math-inline">a+b</span>',
        ),
        await html(
          "a `pre` around inline math stays",
          '<pre><code class="math-inline">x</code></pre>',
        ),
        await html("text across elements", '<span class="math-inline">a<b>+</b>b</span>'),
        await html(
          "other code is left alone",
          '<pre><code class="language-js">$x$</code></pre>',
        ),
        await html(
          "entities are decoded first",
          '<span class="math-inline">a &lt; b &amp;&amp; c</span>',
        ),
        await html(
          "a parse error renders in red",
          '<span class="math-inline">\\alpa</span>',
        ),
        await html("an unclosed group", '<span class="math-inline">\\frac{1}{</span>'),
      ],
    },
    {
      description: "options",
      options: {
        macros: { "\\RR": "\\mathbb{R}" },
        errorColor: "orange",
        output: "html",
      },
      tests: [
        await html("macros, HTML only", '<span class="math-inline">\\RR</span>', {
          macros: { "\\RR": "\\mathbb{R}" },
          errorColor: "orange",
          output: "html",
        }),
        await html("the error color", '<span class="math-inline">\\alpa</span>', {
          macros: { "\\RR": "\\mathbb{R}" },
          errorColor: "orange",
          output: "html",
        }),
      ],
    },
  ]
  const messages = [
    await createMessages(
      "a parse error is reported at its element",
      '<p><span class="math-inline">\\alpa</span></p>',
    ),
    await createMessages(
      "a fence is reported at its `code`",
      '<pre>\n<code class="language-math">\\frac{</code></pre>',
    ),
  ]
  writeGroups({
    path: OUTPUT,
    repository: REHYPE_KATEX.url,
    groups: [
      ...groups.map(group => ({
        description: group.description,
        comment: SORTED,
        source: rehypeKatexSource(),
        pipeline: createPipeline(group.options),
        tests: group.tests,
      })),
      {
        description: "messages",
        source: rehypeKatexSource(),
        pipeline: createPipeline({}),
        tests: messages,
      },
    ],
  })
}

function rehypeKatexSource() {
  return {
    repository: REHYPE_KATEX.url,
    version: REHYPE_KATEX.version,
    commit: REHYPE_KATEX.commit,
    path: "packages/rehype-katex/test.js",
    license: "MIT",
  }
}

function createPipeline(options: KatexOptions) {
  return [
    ["html.parse", { fragment: true }],
    Object.keys(options).length === 0 ? ["html.katex"] : ["html.katex", options],
    ["html.stringify"],
  ]
}

async function createTest(props: {
  description: string
  input: string
  options: KatexOptions
}) {
  const processor = unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeKatex, props.options)
    .use(sortAttributes)
    .use(rehypeStringify)
  const html = String(await processor.process(props.input))
  written += 1
  return { description: props.description, input: props.input, expected: { html } }
}

async function createMessages(description: string, input: string) {
  const processor = unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeKatex)
    .use(rehypeStringify)
  const file = await processor.process(input)
  written += 1
  const messages = file.messages.map(message => ({
    ruleId: message.ruleId,
    source: message.source,
    place: message.place,
  }))
  return { description, input, expected: { messages } }
}

type Node = { type: string; properties?: Record<string, unknown>; children?: Node[] }

function sortAttributes() {
  return (tree: Node) => {
    const stack = [tree]
    for (let node = stack.pop(); node; node = stack.pop()) {
      stack.push(...(node.children ?? []))
      if (node.type !== "element" || !node.properties) continue
      const rank = (name: string) =>
        name === "className" ? "0" : name === "style" ? "1" : `2${name}`
      const entries = Object.entries(node.properties).sort(([left], [right]) =>
        rank(left) < rank(right) ? -1 : 1,
      )
      node.properties = Object.fromEntries(entries)
    }
  }
}

function wrapMath(tex: string, display: boolean) {
  const escaped = tex.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
  return display
    ? `<pre><code class="language-math math-display">${escaped}</code></pre>`
    : `<code class="language-math math-inline">${escaped}</code>`
}
