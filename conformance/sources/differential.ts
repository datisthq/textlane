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

// Fragments where the event engine and the one-pass parser disagreed, found by fuzzing
// both; the pinned reference decides each
const FUZZED_INPUTS: string[] = [
  "+ - <!--~~~===\n\t",
  "+ > ](http://x---\n ",
  "- ~~~\r\n ",
  "- + > + a]\r\n    ",
  "\n\n1. 2) <!--\n\n ",
]
const FUZZED_GFM_INPUTS: string[] = [
  "[^1]: &#x41;# [^1]** https://a.b/c>---<!--\n\t",
  '~~#~<x@y.z>[^1]\r\n[^1]: "t"+ [^1]: \n    ',
  ">*[a]: /uhttp://x**\r\n[^1]: \r\n ",
  "[^1]:  &amp;www.a.b# \n    ",
  "[^1]: https://a.b/c\n\\---\nb\r\n ",
  "[^1]: > \r\n ",
  "_```- \r\n  + >[x] &#x41;\n    ",
  "[^1]: `</div>    &~~~:-**&#x41;<!--\r\n ",
  "<http://a>2)  [a]: /u``2) \n- [^1]: )\n\n\t",
  "[^1]: :-- [ ] (é+ </div>\n\t",
  "[^1]: &amp;\r\n \n",
  "[^1]: <x@y.z>b\n\n\t",
  "[^1]: [^1]:->   \r\n    ",
  "2) ~~~~~~| - |`\n\n\n ",
  " 2) \n\t[ ] </div>`+     <x@y.z>```",
  "[^1]: !```1. \n\t\t",
  "[^1]: <!--]--- \r\n    ",
  "[^1]: <x@y.z>>`\n    ",
  '[^1]: <a href="x">---`<!--\n\n\t',
  "2) [a]: /u]\n\n\t[x] )- ",
  "[^1]: ---1. 't'|-|\n\n ",
  "> 2) 1. |-|\n>",
  "[^1]: [ ] !2) \n*~~~*\n\n ",
  "2) >https://a.b/c*<2) \n ",
  "+ >|-|[a]: /uwww.a.b[\n ",
]
const FUZZED_MATH_INPUTS: string[] = [
  "2) $$\n ",
  "2) > _ \n  ",
  '+ > <a href="x">> &\r\n ',
  "\t\n\n- $$\n ",
  "2) 1. $$\n    ",
  "!\r\n+ $$\n ",
  "- ~~~  ~~~a$$&#x41;---!$$\n  ",
  "1. `$$\n\t#$$<x@y.z>",
  "\n+ <!--1. <div>`!$$\n  ",
]
const FUZZED_DIRECTIVE_INPUTS: string[] = ["2) *  ~~~*{\\# ::a\n2) <http://a>"]
const FUZZED_FRONTMATTER_INPUTS: string[] = [
  "(`---\n- > +++\n  ",
  "&#x41;`- \n\n1. ``+++\n\t(|``&#x41;",
  "+ 2) ~~~#2) ```  ]a: b\n ",
  "1. + ```- ===]a!a: b\n1.  ---\n",
  ">```---\n\t<x@y.z>\n\n\t**<x@y.z>  )",
  "1. >+++\n  ",
  "<\n\n---\n [a]: /u+++\né\t---\n---\n*+ ",
  "  1. ~~~~~~\n\n\t",
  '+++\n"t"\t  ---\n1. - <!--&a: b\n ',
  "+ `\n\t`&#x41;a[a]: /u<div>|---\n---\n",
  '1. 2) ===<a href="x">\n<x@y.z>\n\n    <!--\r\n>',
  "- ~~~+++\n ",
  "+ ~~~&#bb2) &#x41;```a: b\n ",
  "+ <!--!\r\n  ",
  '\\+ \r\n\n+++\n\\\r\n1. >"t"+++\n  ',
  "[a]: /uhttp://x</div>    ((<2) \t",
  '- + 1. ~~~"t"    &é---\n    ',
]
const FUZZED_DEFINITION_LIST_INPUTS: string[] = [
  '\\<a href="x">ba\t\\\r\n2) \n: **```',
  "-->> \t  ~~~\\\r\n(+ + \n: ",
  "\\\n&amp;```<div>_\n: ",
  "(\t\\\n`\n: \n\n>     http://x",
  ":   ---+ <!--_\\\r\n<http://a>a+ )\n: ",
  "`(\\\n&#x41;</div>`'t'\n: (",
  ": <a href=\"x\">\\\r\n] \n:  't'",
  " 2) ] <x@y.z>2) *\n    :   &",
  '---http://x"t"]\\\r\na\n\n: ]*',
  "\\\n]\"t\"**b 't'\n: ",
  "  1. ~~~~~~\n\n\t",
  "> [a]: /u\r\n    + `\n: ",
  "\\\r\n \n: \\",
  "+ `\n\t`&#x41;a[a]: /u<div>|: : ",
  "# _- \n: \t\n\n\t# [a]: /u\n*",
  "\n\n&#x41;\\\r\n[a]: /u**:   _\n: <**(",
  ": ~~~é\n: \n :   ",
  "&amp;<\t> \\\n`[\n:   |",
  "\\\t    \r\n[a]: /u# \n: </div>",
  "===<http://a>\\\n(:   <x@y.z>\n: + <http://a>",
  "\\\r\nba+ |\n: ",
  '"t"\\\n]<http://a>&amp;http://x--- \n: ]',
  '&~~~[a]: /u\\\n"t"*&#x41;: \n: -  ',
  '1. 2) ===<a href="x">\n<x@y.z>\n\n    <!--\r\n>',
  "`    \\\n \\\n: ~~~[``",
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
  const fuzzed = (
    description: string,
    plugin: string | null,
    inputs: string[],
    extensions: Parameters<typeof parseMarkdown>[1],
  ) => ({
    description,
    pipeline: plugin ? [["md.parse"], [plugin]] : [["md.parse"]],
    tests: inputs.map((input, index) => ({
      description: `fragment ${index + 1}`,
      input,
      expected: { tree: parseMarkdown(input, extensions) },
    })),
  })
  const groups = [
    group,
    gfm,
    fuzzed("fuzzed", null, FUZZED_INPUTS, {}),
    fuzzed("fuzzed with GFM", "md.gfm", FUZZED_GFM_INPUTS, { gfm: {} }),
    fuzzed("fuzzed with math", "md.math", FUZZED_MATH_INPUTS, { math: {} }),
    fuzzed("fuzzed with directives", "md.directive", FUZZED_DIRECTIVE_INPUTS, {
      directive: {},
    }),
    fuzzed("fuzzed with frontmatter", "md.frontmatter", FUZZED_FRONTMATTER_INPUTS, {
      frontmatter: null,
    }),
    fuzzed(
      "fuzzed with definition lists",
      "md.definitionList",
      FUZZED_DEFINITION_LIST_INPUTS,
      {
        definitionList: true,
      },
    ),
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
