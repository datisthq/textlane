import { isDeepStrictEqual } from "node:util"
import { existsSync, readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { fromHtml } from "hast-util-from-html"
import { toNlcst as hastToNlcst } from "hast-util-to-nlcst"
import { toNlcst as mdastToNlcst } from "mdast-util-to-nlcst"
import { toString } from "nlcst-to-string"
import { ParseEnglish } from "parse-english"
import { ParseLatin } from "parse-latin"
import { VFile } from "vfile"
import { writeGroups } from "../actions/group/write.ts"
import { parseMarkdown } from "../actions/markdown/render.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const TESTS = resolve(import.meta.dirname, "..", "tests")
const LATIN = {
  url: "https://github.com/wooorm/parse-latin",
  commit: "35dc1554f70e0e870d05130d265af6d6883a5a5f",
}
const ENGLISH = {
  url: "https://github.com/wooorm/parse-english",
  commit: "d50352f9fac7cf71871bf20ed9f2472430924393",
}
const MDAST = {
  url: "https://github.com/syntax-tree/mdast-util-to-nlcst",
  commit: "e05a1eddef74b372d4b50b6cb8e73ec91bb8ef79",
}
const HAST = {
  url: "https://github.com/syntax-tree/hast-util-to-nlcst",
  commit: "9da6a4283200713e3d5050491031e1ef2992c743",
}

type Repository = { url: string; commit: string }

let written = 0
importParser({
  repository: LATIN,
  file: "parse-latin",
  pipeline: [["prose.parse"]],
  parse: input => new ParseLatin().parse(input),
})
importParser({
  repository: ENGLISH,
  file: "parse-english",
  pipeline: [["prose.parse", { language: "english" }]],
  parse: input => new ParseEnglish().parse(input),
})
importMdastBridge()
importHastBridge()
console.log(`prose parse: ${written} cases`)

// A fixture is the tree of one input, and nlcst keeps every character, so the input is the tree's
// text; fixtures of the sentence and paragraph tokenizers are not reachable through a pipeline
function importParser(props: {
  repository: Repository
  file: string
  pipeline: unknown[]
  parse: (input: string) => object
}) {
  const root = join(
    fetchSource({ repository: props.repository.url, commit: props.repository.commit }),
    "test",
    "fixture",
  )
  const tests: object[] = []
  let skipped = 0
  for (const name of readdirSync(root)
    .filter(file => file.endsWith(".json"))
    .sort()) {
    const fixture = JSON.parse(readFileSync(join(root, name), "utf8"))
    if (fixture.type !== "RootNode") {
      skipped += 1
      continue
    }
    const input = toString(fixture)
    const tree = JSON.parse(JSON.stringify(props.parse(input)))
    if (!isDeepStrictEqual(tree, fixture))
      console.log(`  ${props.file} fixture ${name} drifts from its snapshot`)
    tests.push({ description: name.replace(/\.json$/, ""), input, expected: { tree } })
    written += 1
  }
  if (skipped > 0) console.log(`  ${props.file}: skipped ${skipped} tokenizer fixtures`)
  writeGroups({
    path: join(TESTS, "prose", `${props.file}.json`),
    repository: props.repository.url,
    groups: [
      {
        description: `${props.file} fixtures`,
        source: {
          repository: props.repository.url,
          commit: props.repository.commit,
          path: "test/fixture",
          license: "MIT",
        },
        pipeline: props.pipeline,
        tests,
      },
    ],
  })
}

// input.md with options.json: `ignore` and `source` are md.toProse's, the `useRemark*` flags the
// syntax the Markdown is parsed with
function importMdastBridge() {
  const root = join(
    fetchSource({ repository: MDAST.url, commit: MDAST.commit }),
    "test",
    "fixtures",
  )
  const groups = new Map<string, { pipeline: unknown[]; tests: object[] }>()
  for (const name of readdirSync(root).sort()) {
    const optionsPath = join(root, name, "options.json")
    const config: Record<string, unknown> = existsSync(optionsPath)
      ? JSON.parse(readFileSync(optionsPath, "utf8"))
      : {}
    const options = Object.fromEntries(
      Object.entries(config).filter(([key]) => key === "ignore" || key === "source"),
    )
    const input = readFileSync(join(root, name, "input.md"), "utf8")
    const gfm = config.useRemarkGfm === true
    const frontmatter = config.useRemarkFrontmatter === true
    const mdast = parseMarkdown(input, {
      ...(gfm ? { gfm: {} } : {}),
      ...(frontmatter ? { frontmatter: null } : {}),
    })
    // A JSON tree is valid mdast; the utility types its nodes, not plain objects
    const tree = JSON.parse(
      JSON.stringify(
        Reflect.apply(mdastToNlcst, undefined, [
          mdast,
          new VFile(input),
          ParseLatin,
          options,
        ]),
      ),
    )
    if (
      !isDeepStrictEqual(
        tree,
        JSON.parse(readFileSync(join(root, name, "output.json"), "utf8")),
      )
    )
      console.log(`  mdast-util-to-nlcst fixture ${name} drifts from its snapshot`)
    const pipeline = [
      ["md.parse"],
      ...(gfm ? [["md.gfm"]] : []),
      ...(frontmatter ? [["md.frontmatter"]] : []),
      Object.keys(options).length > 0 ? ["md.toProse", options] : ["md.toProse"],
    ]
    const key = JSON.stringify(pipeline)
    const group = groups.get(key) ?? { pipeline, tests: [] }
    group.tests.push({ description: `fixture ${name}`, input, expected: { tree } })
    groups.set(key, group)
    written += 1
  }
  const source = {
    repository: MDAST.url,
    commit: MDAST.commit,
    path: "test/fixtures",
    license: "MIT",
  }
  writeGroups({
    path: join(TESTS, "md", "to-prose.json"),
    repository: MDAST.url,
    groups: [...groups.values()].map((group, index) => ({
      description: `mdast-util-to-nlcst fixtures${index === 0 ? "" : ` (${index + 1})`}`,
      source,
      ...group,
    })),
  })
}

// input.html parsed as a document, as the utility's own harness does
function importHastBridge() {
  const root = join(
    fetchSource({ repository: HAST.url, commit: HAST.commit }),
    "test",
    "fixtures",
  )
  const tests = readdirSync(root)
    .sort()
    .map(name => {
      const input = readFileSync(join(root, name, "input.html"), "utf8")
      const file = new VFile(input)
      const tree = JSON.parse(
        JSON.stringify(hastToNlcst(fromHtml(file), file, ParseLatin)),
      )
      if (
        !isDeepStrictEqual(
          tree,
          JSON.parse(readFileSync(join(root, name, "output.json"), "utf8")),
        )
      )
        console.log(`  hast-util-to-nlcst fixture ${name} drifts from its snapshot`)
      written += 1
      return { description: `fixture ${name}`, input, expected: { tree } }
    })
  writeGroups({
    path: join(TESTS, "html", "to-prose.json"),
    repository: HAST.url,
    groups: [
      {
        description: "hast-util-to-nlcst fixtures",
        source: {
          repository: HAST.url,
          commit: HAST.commit,
          path: "test/fixtures",
          license: "MIT",
        },
        pipeline: [["html.parse"], ["html.toProse"]],
        tests,
      },
    ],
  })
}
