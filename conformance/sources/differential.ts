import { writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { toHast } from "mdast-util-to-hast"
import type { Options as ToHastOptions } from "mdast-util-to-hast"
import { parseMarkdown } from "../actions/markdown/render.ts"

const OUTPUT = join(
  resolve(import.meta.dirname, "..", "tests", "md"),
  "differential.json",
)

// Inputs where md.parse once differed from mdast-util-from-markdown, found by fuzzing
// both with random fragments; each expectation is the pinned reference's tree
const INPUTS: { description: string; input: string }[] = [
  { description: "a tab after a block quote marker", input: ">\ta" },
  { description: "a tab after a block quote marker before HTML", input: ">\t<div>" },
  { description: "a lazy tilde line ends a list item's paragraph", input: "- a\n~" },
  { description: "a lazy backtick line at the end", input: "> a\n`" },
  { description: "a hard break before a lazy HTML line", input: "> a\\\n<" },
  { description: "an empty item after open indented code", input: "-     a\n-" },
  { description: "an HTML item after a paragraph", input: "a\n* <a>" },
  { description: "an ATX heading with inner number signs", input: "# a # b ##" },
  { description: "an indented ATX heading", input: "  ## a" },
  { description: "a fence at the end inside a block quote", input: "> ```\n" },
  { description: "a nested empty item on one line", input: "b c\n1) 1. " },
  { description: "a setext heading after a definition", input: "[a]: b\nc\n---" },
  { description: "a dash underline after only a definition", input: "[a]: b\n---" },
  { description: "an equals underline after only a definition", input: "[a]: b\n===" },
  {
    description: "an indented dash underline after a definition",
    input: "\r\n[a]: b\r\n  ---",
  },
  { description: "a tab after a list marker widens the prefix", input: "-\t```\n  a" },
  { description: "a nested empty item before a sibling", input: "1) 1)   \n1) b c" },
  {
    description: "an empty block quote item before indented code",
    input: "1. >\n        a",
  },
  {
    description: "an empty block quote before a sibling item",
    input: "> + w\n>\n> + x\n",
  },
  {
    description: "indented code cannot interrupt after a closed list",
    input: "-\na\n    ```",
  },
  { description: "a fence at the end after a closed block quote", input: "> a\n~~~\n" },
  { description: "an unclosed fence before a sibling item", input: "- ```\n- b" },
  { description: "unclosed HTML before a sibling item", input: "- <!--\n- b" },
  { description: "a nested empty block quote at the end", input: "> >\n>" },
  { description: "code spans after an unclosed longer run", input: "```x`a`b`c`" },
  { description: "a byte order mark is not counted", input: "\uFEFF# a\nb" },
  { description: "only the first byte order mark is dropped", input: "\uFEFF\uFEFFa" },
  {
    description: "an equals line after a definition starts a paragraph",
    input: "[x]: /u\n===\n[a]: b",
  },
  {
    description: "a heading after a rejected underline starts at its text",
    input: "[a]: b\n===\nc\n===",
  },
  {
    description: "a setext heading in an item takes no lazy line",
    input: "- d\n  ---\n-->",
  },
  { description: "emphasis right after a block quote marker", input: ">***a-****" },
]

// Cases that differed only with GFM on
const GFM_INPUTS: { description: string; input: string }[] = [
  {
    description: "a list item after a possible table head takes indented code",
    input: "c\n*     y",
  },
  {
    description: "strikethrough right after a block quote marker",
    input: ">***~~a~~****",
  },
]

// Directives have no mdast-util-to-hast handler, so its unknown handler and `data` apply
const DIRECTIVE_INPUTS: { description: string; input: string }[] = [
  { description: "a container directive", input: ":::note\nalpha\n:::" },
  {
    description: "a container directive with a label and attributes",
    input: ":::note[Bravo]{.charlie #delta}\necho\n:::",
  },
  { description: "nested container directives", input: "::::a\n:::b\nc\n:::\n::::" },
  { description: "a leaf directive", input: "::video[Foxtrot]{id=golf}" },
  { description: "a leaf directive without a label", input: "::hotel" },
  { description: "a text directive", input: "india :abbr[HTML]{title=juliett} kilo" },
]

type TreeCase = { description: string; tree: object }

const DIRECTIVE_TREES: TreeCase[] = [
  {
    description: "a container directive with `data.hName` and `data.hProperties`",
    tree: {
      type: "containerDirective",
      name: "note",
      attributes: {},
      data: { hName: "aside", hProperties: { className: ["note"] } },
      children: [{ type: "paragraph", children: [{ type: "text", value: "lima" }] }],
    },
  },
  {
    description: "a leaf directive with `data.hChildren`",
    tree: {
      type: "leafDirective",
      name: "video",
      attributes: { id: "mike" },
      data: {
        hName: "video",
        hChildren: [{ type: "text", value: "november" }],
      },
      children: [{ type: "text", value: "oscar" }],
    },
  },
  {
    description: "a text directive with `data.hName` in a paragraph",
    tree: {
      type: "paragraph",
      children: [
        { type: "text", value: "papa " },
        {
          type: "textDirective",
          name: "abbr",
          attributes: { title: "quebec" },
          data: { hName: "abbr", hProperties: { title: "quebec" } },
          children: [{ type: "text", value: "romeo" }],
        },
      ],
    },
  },
]

const PASS_THROUGH_TREES: TreeCase[] = [
  {
    description: "a text directive passed through",
    tree: {
      type: "paragraph",
      children: [
        {
          type: "textDirective",
          name: "sierra",
          attributes: { tango: "uniform" },
          data: { hName: "span" },
          children: [{ type: "emphasis", children: [{ type: "text", value: "victor" }] }],
        },
      ],
    },
  },
]

importDifferential()

// textlane's own cases, not ported from upstream: groups with no source
function importDifferential() {
  const group = {
    description: "differential",
    pipeline: [["md.parse"]],
    tests: INPUTS.map(test => ({
      ...test,
      expected: { tree: parseMarkdown(test.input) },
    })),
  }
  const directives = {
    description: "directive to hast",
    pipeline: [["md.parse"], ["md.directive"], ["md.toHtml"]],
    tests: DIRECTIVE_INPUTS.map(test => ({
      ...test,
      expected: { tree: convertTree(parseMarkdown(test.input, { directive: {} })) },
    })),
  }
  const gfm = {
    description: "differential with GFM",
    pipeline: [["md.parse"], ["md.gfm"]],
    tests: GFM_INPUTS.map(test => ({
      ...test,
      expected: { tree: parseMarkdown(test.input, { gfm: {} }) },
    })),
  }
  const groups = [
    group,
    gfm,
    directives,
    createTreeGroup(DIRECTIVE_TREES),
    createTreeGroup(PASS_THROUGH_TREES, { passThrough: ["textDirective"] }),
  ]
  writeFileSync(OUTPUT, `${JSON.stringify(groups, null, 2)}\n`)
  const count = groups.reduce((total, item) => total + item.tests.length, 0)
  console.log(`differential: ${count} cases`)
}

function createTreeGroup(tests: TreeCase[], options?: ToHastOptions) {
  return {
    description: `directive data to hast${options ? ` ${JSON.stringify(options)}` : ""}`,
    pipeline: [options ? ["md.toHtml", options] : ["md.toHtml"]],
    tests: tests.map(test => ({
      ...test,
      expected: { tree: convertTree(test.tree, options) },
    })),
  }
}

// A JSON copy, so `undefined` fields drop out as they do in the runners' JSON
function convertTree(tree: object, options?: ToHastOptions) {
  const copy = JSON.parse(JSON.stringify(tree))
  return JSON.parse(JSON.stringify(toHast(copy, options)))
}
