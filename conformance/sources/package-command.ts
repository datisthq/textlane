import { readFileSync, writeFileSync } from "node:fs"
import { createRequire, stripTypeScriptTypes } from "node:module"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import { simple } from "acorn-walk"
import remarkParse from "remark-parse"
import { unified } from "unified"
import { evaluateNode } from "../actions/node/evaluate.ts"
import { fetchSource } from "../actions/source/fetch.ts"

// npm-to-yarn's types read as CommonJS, so its default import is not callable to tsgo
const convert: (command: string, to: string) => string = createRequire(import.meta.url)(
  "npm-to-yarn",
)
const TESTS = resolve(import.meta.dirname, "..", "tests")
const repository = "https://github.com/nebrelbug/npm-to-yarn"
const commit = "9ad7c2985e52e2f1e87074ca9bd91aef21998488"
const source = {
  repository,
  version: "3.2.0",
  path: "test/index.spec.ts",
  license: "MIT",
}
const NPM_MANAGERS = ["npm", "pnpm", "yarn", "bun", "deno", "nub"]
const EXECUTORS = ["npx", "yarn dlx", "pnpm dlx", "bun x", "nubx"]
const COMMENT =
  "Expected trees come from the pinned npm-to-yarn's `convert`, line by line; for `npm` " +
  "only a line an executor leads converts, since upstream reads any other as yarn's"

type Tree = { type: string; [key: string]: unknown }
let written = 0
const groups: object[] = []

groups.push(...(await readUpstreamGroups()))
groups.push(await npmGroup())
groups.push(...optionGroups())
groups.push(pipGroup())
groups.push(cargoGroup())
writeFileSync(
  join(TESTS, "md", "package-command.json"),
  `${JSON.stringify(groups, null, 2)}\n`,
)
console.log(`package-command: ${written} cases`)

// One group per describe block of npm-to-yarn's spec, its npm commands as inputs; the
// yarn-to-npm block converts the other way, which a `npm` block never does
async function readUpstreamGroups() {
  const path = fetchSource({ repository, commit })
  const text = readFileSync(join(path, "test", "index.spec.ts"), "utf8")
  const program = parse(stripTypeScriptTypes(text), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  const blocks: { title: string; inputs: string[] }[] = []
  for (const statement of program.body) {
    if (statement.type !== "ExpressionStatement") continue
    const call = statement.expression
    if (call.type !== "CallExpression") continue
    const title = evaluateNode(call.arguments[0], new Map())
    if (typeof title !== "string" || title.startsWith("Yarn to NPM")) continue
    const inputs: string[] = []
    simple(call, {
      VariableDeclarator(node) {
        if (node.id.type !== "Identifier" || node.id.name !== "tests") return
        const value = evaluateNode(node.init, new Map())
        if (!Array.isArray(value)) return
        for (const row of value)
          if (Array.isArray(row) && typeof row[0] === "string") inputs.push(row[0])
      },
    })
    blocks.push({ title, inputs: [...new Set(inputs)] })
  }
  const groups = []
  for (const block of blocks) {
    const tests = []
    for (const command of block.inputs) {
      const input = `\`\`\`npm\n${command}\n\`\`\``
      tests.push({ description: command, input, expected: { tree: await render(input) } })
    }
    written += tests.length
    groups.push({
      description: `npm-to-yarn: ${block.title}`,
      comment: COMMENT,
      source,
      pipeline: [
        ["md.parse"],
        ["md.packageCommand", { ecosystems: { npm: NPM_MANAGERS } }],
      ],
      positions: false,
      tests,
    })
  }
  return groups
}

// textlane's own inputs: blocks with several lines, quirks npm-to-yarn keeps, places in
// the tree
async function npmGroup() {
  const inputs: [string, string][] = [
    ["several lines convert each", "```npm\nnpm install\nnpm run build\n```"],
    ["a blank line stays", "```npm\nnpm i a\n\nnpm test\n```"],
    ["a bare npm", "```npm\nnpm\n```"],
    ["a chain with &&", "```npm\ncd app && npm install && npm start\n```"],
    ["pnpm inside a word", "```npm\npnpm install\n```"],
    ["a lone separator", "```npm\nnpm --\n```"],
    ["quoted arguments", "```npm\nnpm install \"a b\" 'c d' e\\ f\n```"],
    ["an initializer at latest", "```npm\nnpm init vite@latest my-app\n```"],
    ["a scoped initializer", "```npm\nnpm create @scope/app\n```"],
    ["cache clean", "```npm\nnpm cache clean --force\n```"],
    ["a pack destination", "```npm\nnpm pack --pack-destination=out\n```"],
    ["list depth", "```npm\nnpm ls --depth=1 --production\n```"],
    ["version bumps", "```npm\nnpm version minor\n```"],
    ["run of a yarn command", "```npm\nnpm run add\n```"],
    ["exec", "```npm\nnpm exec -- eslint .\n```"],
    ["an unknown command", "```npm\nnpm doctor\n```"],
    ["an executor with leading spaces", "```npm\n  npx tsc\n```"],
    ["an executor in a later line", "```npm\nnpm i\nnpx tsc\n```"],
    ["global install", "```npm\nnpm install -g typescript\n```"],
    ["no package lock", "```npm\nnpm install a --no-package-lock\n```"],
    ["in a list item", "- Install:\n\n  ```npm\n  npm i a\n  ```"],
    ["in a block quote", "> ```npm\n> npm i a\n> ```"],
    ["other code stays", "```sh\nnpm i a\n```\n\n```npm\nnpm i a\n```"],
    ["meta is dropped", '```npm title="x"\nnpm i a\n```'],
    ["an empty block", "```npm\n```"],
  ]
  const tests = []
  for (const [description, input] of inputs)
    tests.push({ description, input, expected: { tree: await render(input) } })
  written += tests.length
  return {
    description: "npm blocks, textlane's own cases",
    comment: COMMENT,
    source,
    pipeline: [
      ["md.parse"],
      ["md.packageCommand", { ecosystems: { npm: NPM_MANAGERS } }],
    ],
    positions: false,
    tests,
  }
}

function optionGroups() {
  const element = (name: string, attributes: [string, string][]) => ({
    type: "root",
    children: [jsx(name, attributes)],
  })
  written += 4
  return [
    {
      description: "the default options",
      comment:
        "npm, pnpm, yarn and bun; pip, uv, poetry and pdm; cargo and cargo-binstall",
      pipeline: [["md.parse"], ["md.packageCommand"]],
      positions: false,
      tests: [
        {
          description: "an npm block",
          input: "```npm\nnpm i -D a\n```",
          expected: {
            tree: element("PackageTabs", [
              ["npm", "npm i -D a"],
              ["pnpm", "pnpm add -D a"],
              ["yarn", "yarn add --dev a"],
              ["bun", "bun add --dev a"],
            ]),
          },
        },
        {
          description: "a cargo block",
          input: "```cargo\ncargo install ripgrep\n```",
          expected: {
            tree: element("PackageTabs", [
              ["cargo", "cargo install ripgrep"],
              ["cargo-binstall", "cargo binstall ripgrep"],
            ]),
          },
        },
      ],
    },
    {
      description: "a component and managers of one's own",
      comment:
        "An ecosystem left out keeps its defaults; an empty list leaves its blocks code",
      pipeline: [
        ["md.parse"],
        [
          "md.packageCommand",
          { component: "Tabs", ecosystems: { npm: ["yarn", "npm"], pip: [] } },
        ],
      ],
      positions: false,
      tests: [
        {
          description: "managers in the order given",
          input: "```npm\nnpm i a\n```",
          expected: {
            tree: element("Tabs", [
              ["yarn", "yarn add a"],
              ["npm", "npm i a"],
            ]),
          },
        },
        {
          description: "an ecosystem turned off",
          input: "```pip\npip install a\n```",
          expected: {
            tree: {
              type: "root",
              children: [
                { type: "code", lang: "pip", meta: null, value: "pip install a" },
              ],
            },
          },
        },
      ],
    },
    {
      description: "invalid options",
      pipeline: [["md.parse"], ["md.packageCommand", { ecosystems: { npm: ["pip"] } }]],
      tests: [
        {
          description: "a manager of another ecosystem",
          input: "```npm\nnpm i\n```",
          expected: { error: { code: "invalid-options" } },
        },
      ],
    },
  ]
}

function pipGroup() {
  // [input, pip, uv, poetry, pdm]
  const cases: [string, string, string, string, string, string][] = [
    [
      "add packages",
      "pip install requests",
      "pip install requests",
      "uv add requests",
      "poetry add requests",
      "pdm add requests",
    ],
    [
      "pip3 and versions",
      'pip3 install "httpx>=0.27" rich',
      'pip3 install "httpx>=0.27" rich',
      'uv add "httpx>=0.27" rich',
      'poetry add "httpx>=0.27" rich',
      'pdm add "httpx>=0.27" rich',
    ],
    [
      "python -m pip",
      "python -m pip install textlane",
      "python -m pip install textlane",
      "uv add textlane",
      "poetry add textlane",
      "pdm add textlane",
    ],
    [
      "a dev group",
      "pip install --group dev pytest",
      "pip install --group dev pytest",
      "uv add --dev pytest",
      "poetry add --group dev pytest",
      "pdm add --dev pytest",
    ],
    [
      "another group",
      "pip install --group=docs mkdocs",
      "pip install --group=docs mkdocs",
      "uv add --group docs mkdocs",
      "poetry add --group docs mkdocs",
      "pdm add --dev --group docs mkdocs",
    ],
    [
      "a group alone installs it",
      "pip install --group dev",
      "pip install --group dev",
      "uv sync --group dev",
      "poetry install --with dev",
      "pdm install -G dev",
    ],
    [
      "the project",
      "pip install .",
      "pip install .",
      "uv sync",
      "poetry install",
      "pdm install",
    ],
    [
      "the project editable",
      "pip install -e .",
      "pip install -e .",
      "uv sync",
      "poetry install",
      "pdm install",
    ],
    [
      "an editable path",
      "pip install -e ../lib",
      "pip install -e ../lib",
      "uv add --editable ../lib",
      "poetry add --editable ../lib",
      "pip install -e ../lib",
    ],
    [
      "requirements",
      "pip install -r requirements.txt",
      "pip install -r requirements.txt",
      "uv add -r requirements.txt",
      "pip install -r requirements.txt",
      "pip install -r requirements.txt",
    ],
    [
      "upgrade",
      "pip install -U textlane",
      "pip install -U textlane",
      "uv add --upgrade textlane",
      "pip install -U textlane",
      "pip install -U textlane",
    ],
    [
      "prereleases",
      "pip install --pre textlane",
      "pip install --pre textlane",
      "uv add --prerelease allow textlane",
      "poetry add --allow-prereleases textlane",
      "pdm add --pre textlane",
    ],
    [
      "an unknown flag",
      "pip install --user textlane",
      "pip install --user textlane",
      "pip install --user textlane",
      "pip install --user textlane",
      "pip install --user textlane",
    ],
    [
      "uninstall",
      "pip uninstall -y requests",
      "pip uninstall -y requests",
      "uv remove requests",
      "poetry remove requests",
      "pdm remove requests",
    ],
    ["list", "pip list", "pip list", "uv pip list", "poetry show", "pdm list"],
    [
      "show",
      "pip show textlane",
      "pip show textlane",
      "uv pip show textlane",
      "poetry show textlane",
      "pdm show textlane",
    ],
    ["freeze", "pip freeze", "pip freeze", "uv pip freeze", "pip freeze", "pip freeze"],
    [
      "python runs in the project",
      "python -m textlane --help",
      "python -m textlane --help",
      "uv run python -m textlane --help",
      "poetry run python -m textlane --help",
      "pdm run python -m textlane --help",
    ],
    [
      "a script",
      "python3 main.py",
      "python3 main.py",
      "uv run python3 main.py",
      "poetry run python3 main.py",
      "pdm run python3 main.py",
    ],
    [
      "pipx run",
      "pipx run ruff check .",
      "pipx run ruff check .",
      "uvx ruff check .",
      "pipx run ruff check .",
      "pipx run ruff check .",
    ],
    [
      "uvx",
      "uvx ruff check .",
      "pipx run ruff check .",
      "uvx ruff check .",
      "uvx ruff check .",
      "uvx ruff check .",
    ],
    [
      "pipx install",
      "pipx install textlane",
      "pipx install textlane",
      "uv tool install textlane",
      "pipx install textlane",
      "pipx install textlane",
    ],
    [
      "uv tool uninstall",
      "uv tool uninstall textlane",
      "pipx uninstall textlane",
      "uv tool uninstall textlane",
      "uv tool uninstall textlane",
      "uv tool uninstall textlane",
    ],
    ["another command", "pytest -q", "pytest -q", "pytest -q", "pytest -q", "pytest -q"],
  ]
  const managers = ["pip", "uv", "poetry", "pdm"]
  const tests = cases.map(([description, input, ...outputs]) => ({
    description,
    input: `\`\`\`pip\n${input}\n\`\`\``,
    expected: {
      tree: {
        type: "root",
        children: [
          jsx(
            "PackageTabs",
            managers.map((manager, index): [string, string] => [
              manager,
              outputs[index] ?? "",
            ]),
          ),
        ],
      },
    },
  }))
  tests.push({
    description: "several lines",
    input: "```pip\npip install textlane\npython -m textlane\n```",
    expected: {
      tree: {
        type: "root",
        children: [
          jsx("PackageTabs", [
            ["pip", "pip install textlane\npython -m textlane"],
            ["uv", "uv add textlane\nuv run python -m textlane"],
            ["poetry", "poetry add textlane\npoetry run python -m textlane"],
            ["pdm", "pdm add textlane\npdm run python -m textlane"],
          ]),
        ],
      },
    },
  })
  written += tests.length
  return {
    description: "pip blocks",
    comment:
      "textlane's own conversions, with no upstream: pip's syntax for uv, poetry and pdm; " +
      "a line a manager has no equivalent for stays as written",
    pipeline: [["md.parse"], ["md.packageCommand"]],
    positions: false,
    tests,
  }
}

function cargoGroup() {
  // [input, cargo-binstall]
  const cases: [string, string, string][] = [
    ["install", "cargo install ripgrep", "cargo binstall ripgrep"],
    [
      "a version",
      "cargo install textlane --version 1.2.0",
      "cargo binstall textlane --version 1.2.0",
    ],
    [
      "a version attached",
      "cargo install textlane --version=1.2.0 --locked",
      "cargo binstall textlane --version 1.2.0 --locked",
    ],
    [
      "a crate at a version",
      "cargo install textlane@1.2.0",
      "cargo binstall textlane@1.2.0",
    ],
    [
      "a target",
      "cargo install bat --target x86_64-unknown-linux-musl",
      "cargo binstall bat --targets x86_64-unknown-linux-musl",
    ],
    ["force", "cargo install -f bat", "cargo binstall --force bat"],
    ["a flag binstall lacks", "cargo install --path .", "cargo install --path ."],
    ["features", "cargo install bat --features x", "cargo install bat --features x"],
    ["add stays", "cargo add textlane", "cargo add textlane"],
    ["no crate", "cargo install", "cargo install"],
  ]
  const tests = cases.map(([description, input, binstall]) => ({
    description,
    input: `\`\`\`cargo\n${input}\n\`\`\``,
    expected: {
      tree: {
        type: "root",
        children: [
          jsx("PackageTabs", [
            ["cargo", input],
            ["cargo-binstall", binstall],
          ]),
        ],
      },
    },
  }))
  written += tests.length
  return {
    description: "cargo blocks",
    comment:
      "textlane's own conversions, with no upstream: `cargo install` for cargo-binstall",
    pipeline: [["md.parse"], ["md.packageCommand"]],
    positions: false,
    tests,
  }
}

// The reference transform: remark-parse's tree, each `npm` block as `md.packageCommand`
// makes it, through the pinned npm-to-yarn
async function render(input: string) {
  const walk = (node: object): object => {
    if ("type" in node && node.type === "code" && "lang" in node && node.lang === "npm")
      return jsx("PackageTabs", attributesFor(String("value" in node ? node.value : "")))
    const { position: _, ...copy } = { position: undefined, ...node }
    if ("children" in node && Array.isArray(node.children))
      return { ...copy, children: node.children.map(walk) }
    return copy
  }
  return walk(unified().use(remarkParse).parse(input))
}

function attributesFor(value: string): [string, string][] {
  return NPM_MANAGERS.map(manager => [
    manager,
    value
      .split("\n")
      .map(line => convertLine(line, manager))
      .join("\n"),
  ])
}

function convertLine(line: string, manager: string) {
  const start = line.trimStart()
  const executor = EXECUTORS.some(e => start === e || start.startsWith(`${e} `))
  if (manager === "npm" && !executor) return line
  return convert(line, manager)
}

function jsx(name: string, attributes: [string, string][]): Tree {
  return {
    type: "mdxJsxFlowElement",
    name,
    attributes: attributes.map(([name, value]) => ({
      type: "mdxJsxAttribute",
      name,
      value,
    })),
    children: [],
    data: { _mdxExplicitJsx: true },
  }
}
