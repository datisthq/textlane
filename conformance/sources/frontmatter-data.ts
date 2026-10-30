import { join, resolve } from "node:path"
import remarkFrontmatter from "remark-frontmatter"
import remarkParse from "remark-parse"
import remarkStringify from "remark-stringify"
import { unified } from "unified"
import { VFile } from "vfile"
import { matter } from "vfile-matter"
import { writeGroups } from "../actions/group/write.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "frontmatter.json")
const REPOSITORY = "https://github.com/vfile/vfile-matter"
const SOURCE = {
  repository: REPOSITORY,
  version: "5.0.1",
  path: "lib/index.js",
  license: "MIT",
}
const COMMENT =
  "textlane's own extension: md.frontmatter's `data` reads the first yaml node as vfile-matter reads the file, on the pinned yaml package"

const DATA: [string, string][] = [
  ["a mapping", "---\ntitle: Hello\nauthor: Ada\n---\n\n# Hello"],
  ["nested mappings", "---\na:\n  b:\n    c: d\n  e: f\n---"],
  [
    "block and flow sequences",
    "---\ntags:\n  - a\n  - b\nflow: [1, two, {three: 3}]\n---",
  ],
  ["integers", "---\na: 1\nb: -2\nc: +3\nd: 0o17\ne: 0x1F\nf: 007\n---"],
  ["floats", "---\na: 1.5\nb: -.5\nc: 1e3\nd: 2.\ne: 1.0\nf: 6.02E+23\n---"],
  [
    "large integers lose precision as numbers",
    "---\na: 12345678901234567890\nb: 9007199254740993\n---",
  ],
  ["infinities and not-a-number become null", "---\na: .inf\nb: -.Inf\nc: .NaN\n---"],
  ["booleans", "---\na: true\nb: False\nc: TRUE\nd: yes\ne: on\n---"],
  ["nulls", "---\na: null\nb: ~\nc:\nd: NULL\ne: Null\n---"],
  [
    "quoted strings stay strings",
    "---\na: \"1\"\nb: 'true'\nc: \"null\"\nd: \"a\\tb\\u00e9\"\ne: 'it''s'\n---",
  ],
  [
    "dates stay strings",
    "---\ndate: 2024-01-02\ntime: 2001-12-14t21:59:43.10-05:00\n---",
  ],
  [
    "literal and folded block scalars",
    "---\nliteral: |\n  a\n  b\nfolded: >\n  a\n  b\nkeep: |+\n  c\n\nstrip: >-\n  d\n---",
  ],
  [
    "multi-line plain and quoted scalars",
    '---\nplain: a\n  b\n\n  c\nquoted: "x\n  y"\n---',
  ],
  [
    "anchors and aliases",
    "---\nbase: &base\n  a: 1\n  b: [2, 3]\ncopy: *base\nlist:\n  - &x hi\n  - *x\n---",
  ],
  [
    "a redefined anchor resolves to the latest",
    "---\na: &x 1\nb: *x\nc: &x 2\nd: *x\n---",
  ],
  [
    "merge keys are plain keys in YAML 1.2",
    "---\nbase: &b {a: 1}\nmerged:\n  <<: *b\n  c: 2\n---",
  ],
  [
    "keys of other types become strings",
    "---\n1: a\n1.5: b\ntrue: c\n~: d\n0x10: e\n.inf: f\n---",
  ],
  ["integer keys come first", "---\nb: 1\n2: x\na: 2\n1: y\n---"],
  ["a string and a number key alike", '---\n"1": a\n1: b\n---'],
  [
    "explicit tags",
    '---\na: !!str 123\nb: !!int "42"\nc: !!float 1\nd: !custom value\ne: !!int abc\nf: ! 12\n---',
  ],
  ["comments", "---\n# a comment\na: 1 # trailing\n---"],
  ["a top-level sequence", "---\n- a\n- b\n---"],
  ["a top-level string", "---\nhello\n---"],
  ["a top-level number", "---\n42\n---"],
  ["a falsy top-level value becomes an empty object", "---\nfalse\n---"],
  ["a zero top-level value becomes an empty object", "---\n0\n---"],
  ["an empty block", "---\n---\n\ntext"],
  ["a block of only comments", "---\n# nothing\n---"],
  ["no frontmatter", "# Hello\n\nworld"],
  ["an empty document", ""],
  ["a thematic break is not frontmatter", "---\n\ntext"],
  ["CRLF line endings", "---\r\na: 1\r\nb: 2\r\n---\r\n\r\ntext"],
  ["unicode", "---\nтитул: Привет\nemoji: 😀\n---"],
]

const ERRORS: [string, string][] = [
  ["a duplicate key", "---\na: 1\na: 2\n---"],
  ["a duplicate key spelled differently", "---\n1: a\n1.0: b\n---"],
  ["an unclosed flow sequence", "---\na: [1, 2\n---"],
  ["a bad indentation", "---\na:\n  b: 1\n c: 2\n---"],
  ["an undefined alias", "---\na: *nope\n---"],
  ["a tab as indentation", "---\na:\n\tb: 1\n---"],
  [
    "exponential aliases",
    "---\na: &a [x, x, x, x, x, x, x, x, x, x]\nb: &b [*a, *a, *a, *a, *a, *a, *a, *a, *a, *a]\nc: &c [*b, *b, *b, *b, *b, *b, *b, *b, *b, *b]\nd: [*c, *c, *c, *c, *c, *c, *c, *c, *c, *c]\n---",
  ],
  [
    "many aliases of one scalar",
    `---\na: &a x\nb: [${Array(100).fill("*a").join(", ")}]\n---`,
  ],
]

const STRIP: [string, string][] = [
  ["frontmatter before a heading", "---\ntitle: Hello\n---\n\n# Hello"],
  ["frontmatter alone", "---\na: 1\n---"],
  ["no frontmatter", "# Hello"],
]

// Messages a case keeps: yaml's own, which textlane words the same; parse errors differ
const KEPT_MESSAGES = [
  "Map keys must be unique",
  "Excessive alias count indicates a resource exhaustion attack",
]

let written = 0
const groups = [
  {
    description: "vfile-matter: data",
    comment: COMMENT,
    source: SOURCE,
    pipeline: [["md.parse"], ["md.frontmatter", { data: true }], ["md.stringify"]],
    tests: await Promise.all(
      DATA.map(([description, input]) => renderData(description, input)),
    ),
  },
  {
    description: "vfile-matter: invalid YAML",
    comment: `${COMMENT}; every failure is \`invalid-frontmatter\`, with yaml's message where textlane words it the same`,
    source: SOURCE,
    pipeline: [["md.parse"], ["md.frontmatter", { data: true }], ["md.stringify"]],
    tests: await Promise.all(
      ERRORS.map(([description, input]) => renderError(description, input)),
    ),
  },
  {
    description: "vfile-matter: strip",
    comment: `${COMMENT}; \`strip\` drops the yaml node, where vfile-matter drops the text before parsing`,
    source: SOURCE,
    pipeline: [
      ["md.parse"],
      ["md.frontmatter", { data: true, strip: true }],
      ["md.stringify"],
    ],
    tests: await Promise.all(
      STRIP.map(([description, input]) => renderStrip(description, input)),
    ),
  },
]
writeGroups({ path: OUTPUT, repository: REPOSITORY, groups })
console.log(`frontmatter data: ${written} cases`)

async function renderData(description: string, input: string) {
  const file = new VFile(input)
  matter(file)
  written += 1
  return { description, input, expected: { fileData: toJson(file.data) } }
}

async function renderError(description: string, input: string) {
  try {
    matter(new VFile(input))
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    const message = KEPT_MESSAGES.find(kept => text.startsWith(kept))
    written += 1
    return {
      description,
      input,
      expected: {
        error: { code: "invalid-frontmatter", ...(message ? { message } : {}) },
      },
    }
  }
  throw new Error(`vfile-matter accepted ${JSON.stringify(input)}`)
}

async function renderStrip(description: string, input: string) {
  const file = new VFile(input)
  matter(file, { strip: true })
  const md = String(
    await unified()
      .use(remarkParse)
      .use(remarkFrontmatter)
      .use(remarkStringify)
      .process(file),
  )
  written += 1
  return { description, input, expected: { md } }
}

// What crosses into the suite is JSON: infinities and NaN become null, as JSON.stringify does
function toJson(value: unknown) {
  return JSON.parse(JSON.stringify(value))
}
