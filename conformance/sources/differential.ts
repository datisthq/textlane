import { join, resolve } from "node:path"
import { writeGroups } from "../actions/group/write.ts"
import { parseMarkdown } from "../actions/markdown/render.ts"

const OUTPUT = join(
  resolve(import.meta.dirname, "..", "tests", "md"),
  "differential.json",
)
const REPOSITORY = "https://github.com/datisthq/textlane"

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
]

importDifferential()

function importDifferential() {
  writeGroups({
    path: OUTPUT,
    repository: REPOSITORY,
    groups: [
      {
        description: "differential",
        source: {
          repository: REPOSITORY,
          path: "conformance/sources/differential.ts",
          license: "MIT",
        },
        pipeline: [["md.parse"]],
        tests: INPUTS.map(test => ({
          ...test,
          expected: { tree: parseMarkdown(test.input) },
        })),
      },
    ],
  })
  console.log(`differential: ${INPUTS.length} cases`)
}
