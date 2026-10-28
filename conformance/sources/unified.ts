import { join, resolve } from "node:path"
import type { Node } from "unist"
import { unified } from "unified"
import type { Processor } from "unified"
import { VFile } from "vfile"
import { writeGroups } from "../actions/group/write.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "core")
const REPOSITORY = {
  url: "https://github.com/unifiedjs/unified",
  commit: "dfaf172e8be10a021e375ab05acf595f65f5acf0",
}

declare module "unified" {
  interface Data {
    recorded?: unknown[]
    [key: string]: unknown
  }
}
declare module "vfile" {
  interface DataMap {
    trace: string[]
  }
}

type Operation = [string, ...unknown[]]
type Outcome = "data" | "tree" | "txt" | "trace" | "messages" | "error"
type Case = {
  file: string
  group: string
  test: string
  description: string
  operations: Operation[]
  outcome: Outcome
}
type Options = { mode?: "sync" | "async"; [key: string]: unknown }

const TREE = { type: "root", children: [{ type: "text", value: "alpha" }] }
const TEXT = ["test.parseText"]
const COMPILE = ["test.compileValue"]

// Each case cites the unified test it ports; its expectation comes from running the operations
// through the pinned unified with the JavaScript test plugins below
const CASES: Case[] = [
  // use
  {
    file: "use",
    group: "use: ignored values",
    test: "should ignore missing values",
    description: "null is ignored",
    operations: [["use", null], ["freeze"]],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: ignored values",
    test: "should ignore missing values",
    description: "an empty list is ignored",
    operations: [["use", []], ["freeze"]],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: invalid values",
    test: "should throw when given invalid values (`false`)",
    description: "false",
    operations: [["use", false]],
    outcome: "error",
  },
  {
    file: "use",
    group: "use: invalid values",
    test: "should throw when given invalid values (`true`)",
    description: "true",
    operations: [["use", true]],
    outcome: "error",
  },
  {
    file: "use",
    group: "use: invalid values",
    test: "should throw when given invalid values (`1`)",
    description: "a number",
    operations: [["use", 1]],
    outcome: "error",
  },
  {
    file: "use",
    group: "use: invalid values",
    test: "should throw when given an empty preset",
    description: "an empty preset",
    operations: [["use", {}]],
    outcome: "error",
  },
  {
    file: "use",
    group: "use: invalid values",
    test: "should throw when given invalid values",
    description: "an unknown plugin name",
    operations: [["use", "test.missing"]],
    outcome: "error",
  },
  {
    file: "use",
    group: "use: plugins",
    test: "should support a plugin",
    description: "a plugin attaches with its options on freeze",
    operations: [["use", "test.recordOptions", { alpha: true }], ["freeze"]],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: plugins",
    test: "should support a plugin w/o options",
    description: "a plugin without options",
    operations: [["use", "test.recordOptions"], ["freeze"]],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: plugins",
    test: "should support a list of plugins",
    description: "a list of plugins",
    operations: [
      [
        "use",
        [
          ["test.recordOptions", { a: 1 }],
          ["test.setData", { key: "b", value: 2 }],
        ],
      ],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: plugins",
    test: "should support a list of tuples",
    description: "a list of plugins with options",
    operations: [
      [
        "use",
        [
          ["test.setData", { key: "a", value: 1 }],
          ["test.setData", { key: "b", value: [1, 2] }],
        ],
      ],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: presets",
    test: "should support presets with plugins",
    description: "a preset with plugins",
    operations: [["use", { plugins: [["test.recordOptions", { a: 1 }]] }], ["freeze"]],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: presets",
    test: "should support presets with settings",
    description: "a preset with settings",
    operations: [["use", { settings: { alpha: true } }], ["freeze"]],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: presets",
    test: "should support presets with settings",
    description: "settings from two presets merge",
    operations: [
      ["use", { settings: { alpha: true } }],
      ["use", { settings: { bravo: 1 } }],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: presets",
    test: "should support presets with plugins and settings",
    description: "a preset with plugins and settings",
    operations: [
      [
        "use",
        {
          plugins: [["test.setData", { key: "a", value: 1 }]],
          settings: { alpha: true },
        },
      ],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: presets",
    test: "should support nested presets",
    description: "a preset inside a preset",
    operations: [
      ["use", { plugins: [{ plugins: [["test.recordOptions", { nested: true }]] }] }],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure objects",
    description: "object options merge, the later winning",
    operations: [
      ["use", "test.recordOptions", { a: 1, b: 1 }],
      ["use", "test.recordOptions", { b: 2, c: 2 }],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure objects",
    description: "a plugin used twice attaches once",
    operations: [
      ["use", "test.recordOptions", { a: 1 }],
      ["use", "test.recordOptions", { a: 2 }],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure to non-objects",
    description: "a scalar replaces object options",
    operations: [
      ["use", "test.recordOptions", { a: 1 }],
      ["use", "test.recordOptions", 1],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure from non-objects",
    description: "object options replace a scalar",
    operations: [
      ["use", "test.recordOptions", 1],
      ["use", "test.recordOptions", { a: 1 }],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure to `true`",
    description: "true re-enables a plugin, without options",
    operations: [
      ["use", "test.recordOptions", { a: 1 }],
      ["use", "test.recordOptions", true],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure to `false`",
    description: "false turns a plugin off",
    operations: [
      ["use", "test.recordOptions", { a: 1 }],
      ["use", "test.recordOptions", false],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure from `false`",
    description: "options turn a disabled plugin back on",
    operations: [
      ["use", "test.recordOptions", false],
      ["use", "test.recordOptions", { a: 1 }],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure presets",
    description: "a preset reconfigures a plugin used before it",
    operations: [
      ["use", "test.recordOptions", { a: 1 }],
      ["use", { plugins: [["test.recordOptions", { b: 2 }]] }],
      ["freeze"],
    ],
    outcome: "data",
  },
  {
    file: "use",
    group: "use: reconfigure",
    test: "should reconfigure lists",
    description: "a list reconfigures a plugin used before it",
    operations: [
      ["use", "test.recordOptions", { a: 1 }],
      ["use", [["test.recordOptions", { a: 2 }]]],
      ["freeze"],
    ],
    outcome: "data",
  },
  // freeze
  {
    file: "freeze",
    group: "freeze: frozen processors",
    test: "should throw when calling `use` on a frozen processor",
    description: "use after freeze",
    operations: [["freeze"], ["use", "test.noop"]],
    outcome: "error",
  },
  {
    file: "freeze",
    group: "freeze: frozen processors",
    test: "should throw when setting data on a frozen processor",
    description: "setting data after freeze",
    operations: [["freeze"], ["data", "alpha", 1]],
    outcome: "error",
  },
  {
    file: "freeze",
    group: "freeze: frozen processors",
    test: "should support getting data on a frozen processor",
    description: "getting data after freeze",
    operations: [
      ["use", "test.setData", { key: "alpha", value: 1 }],
      ["freeze"],
      ["data"],
    ],
    outcome: "data",
  },
  {
    file: "freeze",
    group: "freeze: frozen processors",
    test: "should freeze when parsing",
    description: "parse freezes the processor",
    operations: [
      ["use", ...TEXT],
      ["parse", "a"],
      ["use", "test.noop"],
    ],
    outcome: "error",
  },
  {
    file: "freeze",
    group: "freeze: frozen processors",
    test: "should freeze when processing",
    description: "process freezes the processor",
    operations: [
      ["use", ...TEXT],
      ["use", ...COMPILE],
      ["process", "a", { api: "sync" }],
      ["use", "test.noop"],
    ],
    outcome: "error",
  },
  {
    file: "freeze",
    group: "freeze: attaching",
    test: "should attach plugins once",
    description: "freezing twice attaches once",
    operations: [["use", "test.recordOptions", { a: 1 }], ["freeze"], ["freeze"]],
    outcome: "data",
  },
  {
    file: "freeze",
    group: "freeze: attaching",
    test: "should attach plugins in order",
    description: "plugins attach in the order they were used",
    operations: [
      ["use", "test.setData", { key: "first", value: 1 }],
      ["use", "test.recordOptions", "second"],
      ["freeze"],
    ],
    outcome: "data",
  },
  // data
  {
    file: "data",
    group: "data",
    test: "should set data",
    description: "set then read data",
    operations: [["data", "alpha", "bravo"], ["data"]],
    outcome: "data",
  },
  {
    file: "data",
    group: "data",
    test: "should set data",
    description: "set a nested value",
    operations: [["data", "alpha", { bravo: [1, 2] }], ["data"]],
    outcome: "data",
  },
  {
    file: "data",
    group: "data",
    test: "should overwrite data",
    description: "a later set overwrites",
    operations: [["data", "alpha", 1], ["data", "alpha", 2], ["data"]],
    outcome: "data",
  },
  {
    file: "data",
    group: "data",
    test: "should support settings",
    description: "settings live in data",
    operations: [["data", "settings", { alpha: true }], ["data"]],
    outcome: "data",
  },
  {
    file: "data",
    group: "data",
    test: "should support data from plugins",
    description: "a plugin sets data when it attaches",
    operations: [
      ["use", "test.setData", { key: "alpha", value: true }],
      ["freeze"],
      ["data"],
    ],
    outcome: "data",
  },
  // parse
  {
    file: "parse",
    group: "parse",
    test: "should throw w/o `parser`",
    description: "no parser",
    operations: [["parse", "alpha"]],
    outcome: "error",
  },
  {
    file: "parse",
    group: "parse",
    test: "should support `parser`",
    description: "a parser yields a tree",
    operations: [
      ["use", ...TEXT],
      ["parse", "alpha"],
    ],
    outcome: "tree",
  },
  {
    file: "parse",
    group: "parse",
    test: "should support `parser`",
    description: "an empty document",
    operations: [
      ["use", ...TEXT],
      ["parse", ""],
    ],
    outcome: "tree",
  },
  {
    file: "parse",
    group: "parse",
    test: "should not run transformers",
    description: "parse runs no transformer",
    operations: [
      ["use", ...TEXT],
      ["use", "test.replaceTree", { tree: { type: "root", children: [] } }],
      ["parse", "alpha"],
    ],
    outcome: "tree",
  },
  // stringify
  {
    file: "stringify",
    group: "stringify",
    test: "should throw w/o `compiler`",
    description: "no compiler",
    operations: [["stringify", TREE]],
    outcome: "error",
  },
  {
    file: "stringify",
    group: "stringify",
    test: "should support `compiler`",
    description: "a compiler yields text",
    operations: [
      ["use", ...COMPILE],
      ["stringify", { type: "text", value: "alpha" }],
    ],
    outcome: "txt",
  },
  {
    file: "stringify",
    group: "stringify",
    test: "should support `compiler`",
    description: "a node without a value",
    operations: [
      ["use", ...COMPILE],
      ["stringify", TREE],
    ],
    outcome: "txt",
  },
  {
    file: "stringify",
    group: "stringify",
    test: "should not run transformers",
    description: "stringify runs no transformer",
    operations: [
      ["use", ...COMPILE],
      ["use", "test.fail", { message: "never" }],
      ["stringify", { type: "text", value: "alpha" }],
    ],
    outcome: "txt",
  },
  // run
  ...(["sync", "async"] as const).flatMap(
    api =>
      [
        {
          file: "run",
          group: `run (${api})`,
          test: "should pass through trees w/o transformers",
          description: "no transformer keeps the tree",
          operations: [["run", TREE, { api }]],
          outcome: "tree",
        },
        {
          file: "run",
          group: `run (${api})`,
          test: "should support a sync transformer returning a tree",
          description: "a sync transformer replaces the tree",
          operations: [
            ["use", "test.replaceTree", { tree: { type: "root", children: [] } }],
            ["run", TREE, { api }],
          ],
          outcome: "tree",
        },
        {
          file: "run",
          group: `run (${api})`,
          test: "should support a sync transformer modifying the tree",
          description: "a sync transformer changes the tree",
          operations: [
            ["use", "test.appendNode", { node: { type: "text", value: "bravo" } }],
            ["run", TREE, { api }],
          ],
          outcome: "tree",
        },
        {
          file: "run",
          group: `run (${api})`,
          test: "should support a sync transformer throwing",
          description: "a sync transformer fails",
          operations: [
            ["use", "test.fail", { message: "alpha" }],
            ["run", TREE, { api }],
          ],
          outcome: "error",
        },
        {
          file: "run",
          group: `run (${api})`,
          test: "should run transformers in order",
          description: "stages run in the order they were used",
          operations: [
            ["use", "test.trace:a"],
            ["use", "test.trace:b"],
            ["use", "test.trace:c"],
            ["run", TREE, { api }],
          ],
          outcome: "trace",
        },
        {
          file: "run",
          group: `run (${api})`,
          test: "should attach a plugin once",
          description: "the same stage used twice runs once",
          operations: [
            ["use", "test.trace:a"],
            ["use", "test.trace:a"],
            ["run", TREE, { api }],
          ],
          outcome: "trace",
        },
        {
          file: "run",
          group: `run (${api})`,
          test: "should stop at the first failing transformer",
          description: "a failure stops later stages",
          operations: [
            ["use", "test.fail", { message: "first" }],
            ["use", "test.fail", { message: "second" }],
            ["run", TREE, { api }],
          ],
          outcome: "error",
        },
        {
          file: "run",
          group: `run (${api})`,
          test: "should support messages",
          description: "a stage adds a message",
          operations: [
            [
              "use",
              "test.message",
              { reason: "alpha", ruleId: "bravo", source: "charlie" },
            ],
            ["run", TREE, { api }],
          ],
          outcome: "messages",
        },
      ] satisfies Case[],
  ),
  {
    file: "run",
    group: "run (async)",
    test: "should support an async transformer returning a tree",
    description: "an async transformer replaces the tree",
    operations: [
      [
        "use",
        "test.replaceTree",
        { tree: { type: "root", children: [] }, mode: "async" },
      ],
      ["run", TREE, { api: "async" }],
    ],
    outcome: "tree",
  },
  {
    file: "run",
    group: "run (async)",
    test: "should support an async transformer rejecting",
    description: "an async transformer fails",
    operations: [
      ["use", "test.fail", { message: "alpha", mode: "async" }],
      ["run", TREE, { api: "async" }],
    ],
    outcome: "error",
  },
  {
    file: "run",
    group: "run (async)",
    test: "should run sync and async transformers in order",
    description: "sync and async stages keep their order",
    operations: [
      ["use", "test.trace:a"],
      ["use", "test.trace:b", { mode: "async" }],
      ["use", "test.trace:c"],
      ["run", TREE, { api: "async" }],
    ],
    outcome: "trace",
  },
  {
    file: "run",
    group: "run (sync)",
    test: "should throw when `runSync` finishes async",
    description: "an async stage under the sync API",
    operations: [
      ["use", "test.noop", { mode: "async" }],
      ["run", TREE, { api: "sync" }],
    ],
    outcome: "error",
  },
  {
    file: "run",
    group: "run (sync)",
    test: "should throw when `runSync` finishes async",
    description: "an async replacement under the sync API",
    operations: [
      [
        "use",
        "test.replaceTree",
        { tree: { type: "root", children: [] }, mode: "async" },
      ],
      ["run", TREE, { api: "sync" }],
    ],
    outcome: "error",
  },
  // process
  ...(["sync", "async"] as const).flatMap(
    api =>
      [
        {
          file: "process",
          group: `process (${api})`,
          test: "should throw w/o `parser`",
          description: "no parser",
          operations: [
            ["use", ...COMPILE],
            ["process", "alpha", { api }],
          ],
          outcome: "error",
        },
        {
          file: "process",
          group: `process (${api})`,
          test: "should throw w/o `compiler`",
          description: "no compiler",
          operations: [
            ["use", ...TEXT],
            ["process", "alpha", { api }],
          ],
          outcome: "error",
        },
        {
          file: "process",
          group: `process (${api})`,
          test: "should support a parser and a compiler",
          description: "parse then compile",
          operations: [
            ["use", ...TEXT],
            ["use", ...COMPILE],
            ["process", "alpha", { api }],
          ],
          outcome: "txt",
        },
        {
          file: "process",
          group: `process (${api})`,
          test: "should run transformers",
          description: "a stage changes the output",
          operations: [
            ["use", ...TEXT],
            ["use", "test.replaceTree", { tree: { type: "text", value: "bravo" } }],
            ["use", ...COMPILE],
            ["process", "alpha", { api }],
          ],
          outcome: "txt",
        },
        {
          file: "process",
          group: `process (${api})`,
          test: "should run transformers in order",
          description: "stages run in order",
          operations: [
            ["use", ...TEXT],
            ["use", "test.trace:a"],
            ["use", "test.trace:b"],
            ["use", ...COMPILE],
            ["process", "alpha", { api }],
          ],
          outcome: "trace",
        },
        {
          file: "process",
          group: `process (${api})`,
          test: "should fail when a transformer fails",
          description: "a failing stage",
          operations: [
            ["use", ...TEXT],
            ["use", "test.fail", { message: "alpha" }],
            ["use", ...COMPILE],
            ["process", "alpha", { api }],
          ],
          outcome: "error",
        },
        {
          file: "process",
          group: `process (${api})`,
          test: "should support messages",
          description: "messages reach the file",
          operations: [
            ["use", ...TEXT],
            [
              "use",
              "test.message",
              { reason: "alpha", ruleId: "bravo", source: "charlie" },
            ],
            ["use", ...COMPILE],
            ["process", "alpha", { api }],
          ],
          outcome: "messages",
        },
      ] satisfies Case[],
  ),
  {
    file: "process",
    group: "process (sync)",
    test: "should throw when `processSync` finishes async",
    description: "an async stage under the sync API",
    operations: [
      ["use", ...TEXT],
      ["use", "test.noop", { mode: "async" }],
      ["use", ...COMPILE],
      ["process", "alpha", { api: "sync" }],
    ],
    outcome: "error",
  },
  {
    file: "process",
    group: "process (async)",
    test: "should support async transformers",
    description: "an async stage under the async API",
    operations: [
      ["use", ...TEXT],
      [
        "use",
        "test.replaceTree",
        { tree: { type: "text", value: "bravo" }, mode: "async" },
      ],
      ["use", ...COMPILE],
      ["process", "alpha", { api: "async" }],
    ],
    outcome: "txt",
  },
]

async function importUnified() {
  const byFile = new Map<string, Map<string, object[]>>()
  for (const found of CASES) {
    const outcome = await runOperations(found.operations)
    const key = "error" in outcome ? "error" : found.outcome
    if (key !== found.outcome)
      throw new Error(
        `${found.file}: "${found.description}" expected ${found.outcome}, got ${JSON.stringify(outcome.error)}`,
      )
    const expected = { [key]: outcome[key] }
    const groups = byFile.get(found.file) ?? new Map<string, object[]>()
    byFile.set(found.file, groups)
    groups.set(found.group, [
      ...(groups.get(found.group) ?? []),
      {
        description: found.description,
        comment: `unified test: ${found.test}`,
        operations: found.operations,
        expected,
      },
    ])
  }
  for (const [file, groups] of byFile) {
    writeGroups({
      path: join(OUTPUT, `${file}.json`),
      repository: REPOSITORY.url,
      groups: [...groups].map(([description, tests]) => ({
        description,
        source: {
          repository: REPOSITORY.url,
          commit: REPOSITORY.commit,
          path: `test/${file}.js`,
          license: "MIT",
        },
        pipeline: [],
        tests,
      })),
    })
  }
  console.log(`unified: ${CASES.length} cases in ${byFile.size} files`)
}

// The operations against the pinned unified; every outcome key the last step yields
async function runOperations(
  operations: Operation[],
): Promise<Partial<Record<Outcome, unknown>>> {
  let processor: Processor = unified()
  let outcome: Partial<Record<Outcome, unknown>> = {}
  try {
    for (const [name, ...args] of operations) {
      if (name === "use") {
        // Invalid usables go to unified untyped, so it rejects them as upstream's tests expect
        processor = Reflect.apply(processor.use, processor, [
          toUsable(args[0]),
          ...args.slice(1),
        ])
        outcome = { data: plain(processor.data()) }
      } else if (name === "freeze") {
        processor.freeze()
        outcome = { data: plain(processor.data()) }
      } else if (name === "data") {
        if (args.length > 1) processor.data(String(args[0]), args[1])
        outcome = { data: plain(processor.data()) }
      } else if (name === "parse") {
        outcome = { tree: plain(processor.parse(String(args[0]))) }
      } else if (name === "stringify") {
        outcome = { txt: String(processor.stringify(toNode(args[0]))) }
      } else if (name === "run") {
        const options = isOptions(args[1]) ? args[1] : {}
        const tree = toNode(args[0])
        const file = new VFile()
        const result =
          options.api === "async"
            ? await processor.run(tree, file)
            : processor.runSync(tree, file)
        outcome = describeFile({ file, tree: result })
      } else if (name === "process") {
        const options = isOptions(args[1]) ? args[1] : {}
        const file =
          options.api === "async"
            ? await processor.process(String(args[0]))
            : processor.processSync(String(args[0]))
        outcome = { ...describeFile({ file }), txt: String(file) }
      }
    }
  } catch (error) {
    return { error: toError(error) }
  }
  return outcome
}

function describeFile(props: { file: VFile; tree?: Node }) {
  return {
    ...(props.tree ? { tree: plain(props.tree) } : {}),
    trace: props.file.data.trace ?? [],
    messages: props.file.messages.map(message => ({
      ruleId: message.ruleId ?? "",
      source: message.source ?? "",
      place: plain(message.place ?? null),
    })),
  }
}

// A plugin name, a preset (names inside), a list of `[name, options]`, or any other value passed
// through so unified rejects it as upstream does
function toUsable(value: unknown): unknown {
  if (typeof value === "string") {
    const plugin = PLUGINS[value] ?? createTrace(value)
    if (!plugin) throw new UnknownPluginError(value)
    return plugin
  }
  if (Array.isArray(value)) {
    return value.map(entry =>
      Array.isArray(entry) ? [toUsable(entry[0]), ...entry.slice(1)] : toUsable(entry),
    )
  }
  if (
    typeof value === "object" &&
    value !== null &&
    "plugins" in value &&
    Array.isArray(value.plugins)
  ) {
    return { ...value, plugins: toUsable(value.plugins) }
  }
  return value
}

// `test.trace:<label>` names one plugin per label: unified (and textlane) treat a plugin used
// twice as one plugin reconfigured, so stage order needs distinct plugins
const traces = new Map<string, (this: Processor, options?: unknown) => unknown>()

function createTrace(name: string) {
  if (!name.startsWith("test.trace:")) return undefined
  const label = name.slice("test.trace:".length)
  const existing = traces.get(label)
  if (existing) return existing
  const plugin = function (options?: unknown) {
    return withMode(options, (tree, file) => {
      file.data.trace = [...(file.data.trace ?? []), label]
      return tree
    })
  }
  traces.set(label, plugin)
  return plugin
}

function toNode(value: unknown): Node {
  const node = plain(value)
  if (!isNode(node)) throw new Error(`not a node: ${JSON.stringify(value)}`)
  return node
}

function isNode(value: unknown): value is Node {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    typeof value.type === "string"
  )
}

class UnknownPluginError extends Error {
  constructor(name: string) {
    super(`no plugin named \`${name}\``)
  }
}

// unified's messages onto stable codes; the message stays for a substring match
function toError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error)
  if (error instanceof UnknownPluginError) return { code: "unknown-plugin", message }
  if (/frozen processor/.test(message)) return { code: "frozen-processor", message }
  if (/without `parser`/.test(message)) return { code: "missing-parser", message }
  if (/without `compiler`/.test(message)) return { code: "missing-compiler", message }
  if (/finished async/.test(message)) return { code: "async-in-sync", message }
  if (/usable value|list of plugins|empty preset/.test(message))
    return { code: "invalid-plugin" }
  return { code: "plugin-failed", message }
}

function isOptions(value: unknown): value is Options {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}

function plain<T>(value: T): T {
  return JSON.parse(JSON.stringify(value))
}

function withMode(
  options: unknown,
  transform: (tree: Node, file: VFile) => Node | undefined,
) {
  if (isOptions(options) && options.mode === "async") {
    return async (tree: Node, file: VFile) => transform(tree, file)
  }
  return (tree: Node, file: VFile) => transform(tree, file)
}

// The test plugins, as JavaScript for the reference run; the crate implements the same behaviour
const PLUGINS: Record<string, (this: Processor, options?: unknown) => unknown> = {
  "test.parseText": function () {
    this.parser = document => {
      const node = { type: "text", value: String(document) }
      return node
    }
  },
  "test.compileValue": function () {
    this.compiler = tree =>
      "value" in tree && typeof tree.value === "string" ? tree.value : ""
  },
  "test.recordOptions": function (options) {
    this.data("recorded", [...(this.data("recorded") ?? []), options ?? null])
  },
  "test.setData": function (options) {
    if (isOptions(options) && typeof options.key === "string")
      this.data(options.key, options.value)
  },
  "test.noop": function (options) {
    return withMode(options, () => undefined)
  },
  "test.replaceTree": function (options) {
    return withMode(options, () =>
      isOptions(options) ? toNode(options.tree) : undefined,
    )
  },
  "test.appendNode": function (options) {
    return withMode(options, tree => {
      const node = isOptions(options) ? plain(options.node) : undefined
      if (node && "children" in tree && Array.isArray(tree.children))
        tree.children.push(node)
      return tree
    })
  },
  "test.fail": function (options) {
    return withMode(options, () => {
      throw new Error(isOptions(options) ? String(options.message) : "failed")
    })
  },
  "test.message": function (options) {
    return (_tree: Node, file: VFile) => {
      if (!isOptions(options)) return
      file.message(String(options.reason), {
        ruleId: String(options.ruleId),
        source: String(options.source),
      })
    }
  },
}

// Last, so every class and table above is initialized before the run
await importUnified()
