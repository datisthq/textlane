import { join, resolve } from "node:path"
import rehypeExtractToc from "@stefanprobst/rehype-extract-toc"
import rehypeExternalLinks from "rehype-external-links"
import rehypeParse from "rehype-parse"
import rehypeSlug from "rehype-slug"
import rehypeStringify from "rehype-stringify"
import rehypeUnwrapImages from "rehype-unwrap-images"
import remarkGfm from "remark-gfm"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkStringify from "remark-stringify"
import stripMarkdown from "strip-markdown"
import { createRequire } from "node:module"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"

// remark-heading-id 1.0.1 is CommonJS
const remarkHeadingId = createRequire(import.meta.url)("remark-heading-id")

const TESTS = resolve(import.meta.dirname, "..", "tests")
let written = 0

await importHeadingId()
await importStrip()
await importUnwrapImages()
await importExtractToc()
await importExternalLinks()
console.log(`site plugins: ${written} cases`)

async function importHeadingId() {
  const repository = "https://github.com/imcuttle/remark-heading-id"
  const inputs: [string, string][] = [
    ["an id after a space", "# Hello {#hi}"],
    ["trailing spaces after the id", "# Hello {#hi}   "],
    ["no space before the id", "# Hello{#hi}"],
    ["an id after emphasis", "# Hello *big* {#x}"],
    ["an id after code", "# A `b` {#c}"],
    ["the last braces of two", "## A {#a} {#b}"],
    ["an empty id", "# A {#}"],
    ["an id with a space", "# A {#x y}"],
    ["a setext heading", "A {#s}\n==="],
    ["a heading without an id", "# Plain"],
    ["several headings", "# A {#one}\n\n## B\n\n### C {#three}"],
  ]
  const tests = []
  for (const [description, input] of inputs) {
    const html = String(
      await unified()
        .use(remarkParse)
        .use(remarkHeadingId)
        .use(remarkRehype)
        .use(rehypeStringify)
        .process(input),
    )
    tests.push({ description, input, expected: { html } })
    written += 1
  }
  writeGroups({
    path: join(TESTS, "md", "heading-id.json"),
    repository,
    groups: [
      {
        description: "custom heading ids",
        comment:
          "remark-heading-id's `defaults` option is not ported: html.slug makes ids",
        source: { repository, version: "1.0.1", path: "index.js", license: "MIT" },
        pipeline: [["md.parse"], ["md.headingId"], ["md.toHtml"], ["html.stringify"]],
        tests,
      },
    ],
  })
}

async function importStrip() {
  const repository = "https://github.com/remarkjs/strip-markdown"
  const source = { repository, version: "6.0.0", path: "lib/index.js", license: "MIT" }
  const inputs: [string, string][] = [
    ["a heading", "# Alpha"],
    [
      "emphasis, strong and links",
      "Some *emphasis*, **strong** and [a link](https://example.com).",
    ],
    ["an image takes its alt", "An ![alt text](a.png) image"],
    ["an image without alt takes its title", '![](a.png "Title")'],
    ["code blocks are dropped", "Before\n\n```js\nlet a = 1\n```\n\nAfter"],
    ["inline code keeps its text", "Use `npm i` now"],
    ["HTML is dropped", "a <b>bold</b> c\n\n<div>block</div>"],
    ["a list", "- one\n- two\n  - three"],
    ["a block quote", "> quoted *text*"],
    ["a hard break", "a  \nb"],
    ["a thematic break", "a\n\n***\n\nb"],
    ["a definition and a reference", "[a][b]\n\n[b]: https://example.com"],
  ]
  const gfmInputs: [string, string][] = [
    ["a table is dropped", "| a | b |\n| - | - |\n| 1 | 2 |"],
    ["strikethrough keeps its text", "~~gone~~ here"],
    ["a footnote is dropped", "a[^1]\n\n[^1]: note"],
  ]
  const render = async (input: string, options: object | undefined, gfm: boolean) => {
    const processor = unified().use(remarkParse)
    if (gfm) processor.use(remarkGfm)
    processor.use(stripMarkdown, options).use(remarkStringify)
    written += 1
    return String(await processor.process(input))
  }
  const group = async (
    description: string,
    inputs: [string, string][],
    options: object | undefined,
    gfm: boolean,
  ) => {
    const tests = []
    for (const [name, input] of inputs)
      tests.push({
        description: name,
        input,
        expected: { md: await render(input, options, gfm) },
      })
    const pipeline: unknown[] = [["md.parse"]]
    if (gfm) pipeline.push(["md.gfm"])
    pipeline.push(options ? ["md.strip", options] : ["md.strip"], ["md.stringify"])
    return { description, source, pipeline, tests }
  }
  const groups = [
    await group("Markdown down to its text", inputs, undefined, false),
    await group("GFM down to its text", gfmInputs, undefined, true),
    await group(
      "keep leaves a type as it is",
      [["inline code stays", "Use `npm i` now"]],
      { keep: ["inlineCode"] },
      false,
    ),
    await group(
      "remove drops a type",
      [["lists go", "a\n\n- one\n- two\n\nb"]],
      { remove: ["list"] },
      false,
    ),
  ]
  writeGroups({ path: join(TESTS, "md", "strip.json"), repository, groups })
}

async function importUnwrapImages() {
  const repository = "https://github.com/rehypejs/rehype-unwrap-images"
  const inputs: [string, string][] = [
    ["an image alone", '<p><img src="a.png"></p>'],
    ["images and white space", '<p><img src="a.png"> <img src="b.png">\n</p>'],
    ["an image in a link", '<p><a href="/x"><img src="a.png"></a></p>'],
    ["an image beside text", '<p>Look <img src="a.png"></p>'],
    ["a link without href is not interactive", '<p><a><img src="a.png"></a></p>'],
    ["a link holding text too", '<p><a href="/x"><img src="a.png"> more</a></p>'],
    ["an empty paragraph", "<p></p>"],
    ["an image in a button", '<p><button><img src="a.png"></button></p>'],
    ["nested in a div", '<div><p><img src="a.png"></p></div>'],
    [
      "a link in a link is other content",
      '<p><a href="/x"><a href="/y"><img src="a.png"></a></a></p>',
    ],
  ]
  const tests = []
  for (const [description, input] of inputs) {
    const html = String(
      await unified()
        .use(rehypeParse, { fragment: true })
        .use(rehypeUnwrapImages)
        .use(rehypeStringify)
        .process(input),
    )
    tests.push({ description, input, expected: { html } })
    written += 1
  }
  writeGroups({
    path: join(TESTS, "html", "unwrap-images.json"),
    repository,
    groups: [
      {
        description: "paragraphs of images unwrapped",
        source: { repository, version: "1.0.0", path: "lib/index.js", license: "MIT" },
        pipeline: [
          ["html.parse", { fragment: true }],
          ["html.unwrapImages"],
          ["html.stringify"],
        ],
        tests,
      },
    ],
  })
}

async function importExtractToc() {
  const repository = "https://github.com/stefanprobst/rehype-extract-toc"
  const inputs: [string, string][] = [
    ["nested headings", "<h1>A</h1><h2>B</h2><h3>C</h3><h2>D</h2>"],
    ["a level skipped", "<h1>A</h1><h3>B</h3><h2>C</h2>"],
    ["shallower after deeper", "<h3>A</h3><h1>B</h1><h2>C</h2>"],
    ["siblings at one level", "<h2>A</h2><h2>B</h2><h2>C</h2>"],
    ["text inside elements", "<h2>An <em>emphasized</em> <code>code</code></h2>"],
    ["headings inside sections", "<section><h2>A</h2><div><h3>B</h3></div></section>"],
    ["no headings", "<p>Text</p>"],
    ["duplicate titles", "<h2>Same</h2><h2>Same</h2>"],
  ]
  const tests = []
  for (const [description, input] of inputs) {
    const file = await unified()
      .use(rehypeParse, { fragment: true })
      .use(rehypeSlug)
      .use(rehypeExtractToc)
      .use(rehypeStringify)
      .process(input)
    tests.push({ description, input, expected: { fileData: { toc: file.data.toc } } })
    written += 1
  }
  writeGroups({
    path: join(TESTS, "html", "extract-toc.json"),
    repository,
    groups: [
      {
        description: "headings as a table of contents in the file's data",
        source: { repository, version: "3.0.0", path: "src/index.js", license: "MIT" },
        pipeline: [
          ["html.parse", { fragment: true }],
          ["html.slug"],
          ["html.extractToc"],
          ["html.stringify"],
        ],
        tests,
      },
    ],
  })
}

async function importExternalLinks() {
  const repository = "https://github.com/rehypejs/rehype-external-links"
  const source = { repository, version: "3.0.0", path: "lib/index.js", license: "MIT" }
  const links = [
    '<a href="https://example.com">a</a>',
    '<a href="http://example.com">b</a>',
    '<a href="//cdn.example.com/x">c</a>',
    '<a href="mailto:a@example.com">d</a>',
    '<a href="/docs">e</a>',
    '<a href="#top">f</a>',
    '<a href="ftp://example.com">g</a>',
    '<a href="HTTP://example.com">h</a>',
    '<a href="C:\\path">i</a>',
    "<a>j</a>",
  ].join(" ")
  const groups: { description: string; options?: object; inputs: [string, string][] }[] =
    [
      { description: "defaults", inputs: [["each kind of URL", links]] },
      {
        description: "target and rel as a string",
        options: { target: "_blank", rel: "noopener noreferrer" },
        inputs: [["each kind of URL", links]],
      },
      {
        description: "an empty rel list",
        options: { rel: [] },
        inputs: [["a link", links]],
      },
      {
        description: "an empty rel string",
        options: { rel: "" },
        inputs: [["a link", links]],
      },
      {
        description: "other protocols",
        options: { protocols: ["mailto", "ftp"] },
        inputs: [["each kind of URL", links]],
      },
      {
        description: "more properties",
        options: { properties: { className: ["external"], title: "Leaves the site" } },
        inputs: [["a link", '<a href="https://example.com" title="old">a</a>']],
      },
      {
        description: "content with its properties",
        options: {
          content: { type: "text", value: " (opens in a new tab)" },
          contentProperties: { className: ["sr-only"] },
        },
        inputs: [["a link", '<a href="https://example.com">a</a>']],
      },
      {
        description: "a list of content nodes",
        options: {
          content: [
            { type: "text", value: " " },
            {
              type: "element",
              tagName: "b",
              properties: {},
              children: [{ type: "text", value: "↗" }],
            },
          ],
        },
        inputs: [["a link", '<a href="https://example.com">a</a>']],
      },
    ]
  const writtenGroups = []
  for (const group of groups) {
    const tests = []
    for (const [description, input] of group.inputs) {
      const html = String(
        await unified()
          .use(rehypeParse, { fragment: true })
          .use(rehypeExternalLinks, group.options as never)
          .use(rehypeStringify)
          .process(input),
      )
      tests.push({ description, input, expected: { html } })
      written += 1
    }
    writtenGroups.push({
      description: group.description,
      source,
      pipeline: [
        ["html.parse", { fragment: true }],
        group.options ? ["html.externalLinks", group.options] : ["html.externalLinks"],
        ["html.stringify"],
      ],
      tests,
    })
  }
  writeGroups({
    path: join(TESTS, "html", "external-links.json"),
    repository,
    groups: writtenGroups,
  })
}
