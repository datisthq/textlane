import { mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, relative, resolve } from "node:path"
import { afterAll, beforeAll, describe, expect, it } from "vite-plus/test"
import type { Node, Plugin } from "../index.ts"
import { ADDON_ENV } from "../settings.ts"
import { buildAddon } from "./addon.ts"

// The conformance runner goes through `runCase`; this samples the suite through the
// public API instead, so a mapping the runner never sees cannot drift unnoticed.

const ROOT = resolve(import.meta.dirname, "..", "..")
const TESTS = join(ROOT, "conformance", "tests")
const PENDING: Record<string, string> = JSON.parse(
  readFileSync(join(ROOT, "conformance", "pending.json"), "utf8"),
)
const SAMPLE_PER_FILE = 3
const OUTPUTS = ["html", "md", "js", "txt"]

type Expected = { error?: { code: string } } & Record<string, unknown>
type Test = {
  description: string
  input?: string
  operations?: unknown
  expected: Expected
}
type Group = {
  description: string
  pipeline: [string, unknown?][]
  positions?: boolean
  tests: Test[]
}

let entry: typeof import("../index.ts")
let addonDir = ""

beforeAll(async () => {
  addonDir = mkdtempSync(join(tmpdir(), "textlane-addon-"))
  process.env[ADDON_ENV] = buildAddon(addonDir)
  entry = await import("../index.ts")
}, 600_000)

afterAll(() => {
  if (addonDir) rmSync(addonDir, { recursive: true, force: true })
})

describe("retained memory", () => {
  it("reads back what was set, 256 MB by default", () => {
    expect(entry.retainedMemory()).toBe(256 * 1024 ** 2)
    entry.setRetainedMemory(0)
    expect(entry.retainedMemory()).toBe(0)
    entry.setRetainedMemory(256 * 1024 ** 2)
    expect(() => entry.setRetainedMemory(-1)).toThrow()
  })
})

describe("conformance through the public API", () => {
  for (const file of listFiles(TESTS)) {
    const id = relative(TESTS, file)
      .replace(/\\/g, "/")
      .replace(/\.json$/, "")
    const groups: Group[] = JSON.parse(readFileSync(file, "utf8"))
    const sample = groups
      .flatMap(group => group.tests.map(test => ({ group, test })))
      .filter(({ group, test }) => {
        const groupId = `${id}#${group.description}`
        const ids = [id, groupId, `${groupId}#${test.description}`]
        const pending = Object.keys(PENDING).some(key =>
          key.endsWith("*") ? id.startsWith(key.slice(0, -1)) : ids.includes(key),
        )
        // host plugins need the runner's stand-ins: conformance.unit.ts runs them publicly
        const hosted = group.pipeline.some(
          entry => Array.isArray(entry) && String(entry[0]).startsWith("host:"),
        )
        return test.input !== undefined && !test.operations && !pending && !hosted
      })
      .slice(0, SAMPLE_PER_FILE)
    for (const { group, test } of sample) {
      it(`${id}#${group.description}#${test.description}`, () => runTest(group, test))
    }
  }
})

function runTest(group: Group, test: Test) {
  const plugins: Plugin[] = group.pipeline.map(([name, options]) =>
    options === undefined ? { name } : { name, options },
  )
  const processor = () => entry.textlane().use(plugins)
  const input = test.input ?? ""
  const output = OUTPUTS.find(key => key in test.expected)
  if (test.expected.error) {
    const call = output
      ? () => processor().processSync(input)
      : () => processor().runSync(processor().parse(input), input)
    return expect(call).toThrow(
      expect.objectContaining({ code: test.expected.error.code }),
    )
  }
  if (output) {
    return expect(processor().processSync(input).value).toEqual(test.expected[output])
  }
  if ("tree" in test.expected) {
    const tree: Node = processor().runSync(processor().parse(input), input)
    // A tree read back from JSON keeps unist positions, which a bridge into nlcst cannot
    // place its nodes by: only `process` keeps them in that case
    const bridged = group.pipeline.some(([name]) => name.endsWith(".toNaturalLanguage"))
    if (group.positions === false || bridged) {
      return expect(stripPositions(tree)).toEqual(stripPositions(test.expected.tree))
    }
    return expect(tree).toEqual(test.expected.tree)
  }
  // Messages-only cases whose pipeline has no compiler are the runner's alone
  if (!group.pipeline.some(([name]) => name.endsWith("stringify"))) return
  const messages = processor()
    .processSync(input)
    .messages.map(message => ({
      ruleId: message.ruleId,
      source: message.source,
      place: message.place,
    }))
  expect(messages).toEqual(test.expected.messages)
}

function stripPositions<T>(value: T): T {
  return JSON.parse(
    JSON.stringify(value, (key, item) => (key === "position" ? undefined : item)),
  )
}

function listFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return listFiles(path)
    return entry.name.endsWith(".json") ? [path] : []
  })
}
