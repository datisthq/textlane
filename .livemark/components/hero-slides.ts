export type Lang = "md" | "html" | "js" | "ts" | "txt" | "report"

export interface Stage {
  name: string
  options?: Record<string, unknown>
  /** A JavaScript plugin from npm, run as is between the native stages. */
  host?: boolean
}

export interface Slide {
  id: string
  title: string
  stages: Stage[]
  ports: string[]
  /** The document the pipeline processes; the input pane shows it unless `code` is set. */
  document: string
  code?: string
  input: { label: string; lang: Lang }
  output: { label: string; lang: Lang; text: string }
}

// `.scripts/-test/hero.unit.ts` runs every slide through the addon and compares `output.text`
export const slides: Slide[] = [
  {
    id: "convert",
    title: "Convert",
    stages: [
      { name: "md.parse" },
      { name: "md.gfm" },
      { name: "md.gemoji" },
      { name: "md.toHtml" },
      { name: "html.slug" },
      { name: "html.stringify" },
    ],
    ports: [
      "remark-parse",
      "remark-gfm",
      "remark-gemoji",
      "remark-rehype",
      "rehype-slug",
      "rehype-stringify",
    ],
    document: "# Hello, *lane*\n\n- [x] parse :zap:\n- [ ] ~~rewrite~~",
    input: { label: "input.md", lang: "md" },
    output: {
      label: "output.html",
      lang: "html",
      text: '<h1 id="hello-lane">Hello, <em>lane</em></h1>\n<ul class="contains-task-list">\n<li class="task-list-item"><input type="checkbox" checked disabled> parse ⚡</li>\n<li class="task-list-item"><input type="checkbox" disabled> <del>rewrite</del></li>\n</ul>',
    },
  },
  {
    id: "transform",
    title: "Transform",
    stages: [
      { name: "md.parse" },
      { name: "md.gemoji" },
      { name: "md.smartypants" },
      { name: "md.toc" },
      { name: "md.stringify" },
    ],
    ports: [
      "remark-parse",
      "remark-gemoji",
      "remark-smartypants",
      "remark-toc",
      "remark-stringify",
    ],
    document:
      "# Release notes\n\n## Contents\n\n## What's new\n\n\"Lanes\" -- now *3x faster*... :rocket:\n\n## Upgrading\n\nRun `npm install textlane` -- that's it.",
    input: { label: "input.md", lang: "md" },
    output: {
      label: "output.md",
      lang: "md",
      text: "# Release notes\n\n## Contents\n\n* [What’s new](#whats-new)\n* [Upgrading](#upgrading)\n\n## What’s new\n\n“Lanes” — now *3x faster*… 🚀\n\n## Upgrading\n\nRun `npm install textlane` — that’s it.\n",
    },
  },
  {
    id: "lint",
    title: "Lint",
    stages: [
      { name: "nl.parse" },
      { name: "nl.passive" },
      { name: "nl.repeatedWords" },
      { name: "nl.stringify" },
    ],
    ports: [
      "retext-english",
      "retext-passive",
      "retext-repeated-words",
      "retext-stringify",
    ],
    document: "It was hit by the the ball.",
    input: { label: "input.txt", lang: "txt" },
    output: {
      label: "report",
      lang: "report",
      text: "input.txt\n  1:8-1:11   warning  Unexpected use of the passive voice  hit  retext-passive\n  1:15-1:22  warning  Unexpected repeated `the`, remove one occurrence  the  retext-repeated-words\n\n⚠ 2 warnings",
    },
  },
  {
    id: "plugins",
    title: "Plugins",
    stages: [
      { name: "md.parse" },
      { name: "md.toHtml" },
      { name: "html.slug" },
      { name: "rehypeAutolinkHeadings", host: true },
      { name: "html.stringify" },
    ],
    ports: ["remark-parse", "remark-rehype", "rehype-slug", "rehype-stringify"],
    document: "# Lanes\n\n## Native stages",
    code: 'import rehypeAutolinkHeadings from "rehype-autolink-headings"\n\nconst file = await textlane()\n  .use(md.parse())\n  .use(md.toHtml())\n  .use(html.slug())\n  .use(rehypeAutolinkHeadings)\n  .use(html.stringify())\n  .process("# Lanes\\n\\n## Native stages")',
    input: { label: "main.ts", lang: "ts" },
    output: {
      label: "output.html",
      lang: "html",
      text: '<h1 id="lanes"><a aria-hidden="true" tabindex="-1" href="#lanes"><span class="icon icon-link"></span></a>Lanes</h1>\n<h2 id="native-stages"><a aria-hidden="true" tabindex="-1" href="#native-stages"><span class="icon icon-link"></span></a>Native stages</h2>',
    },
  },
]
