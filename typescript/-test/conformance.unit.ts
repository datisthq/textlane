import { mkdtempSync, readdirSync, readFileSync } from "node:fs"
import { createRequire } from "node:module"
import { tmpdir } from "node:os"
import { join, relative, resolve } from "node:path"
import { afterAll, beforeAll, describe, expect, it } from "vite-plus/test"
import { resolveAddon } from "../actions/addon/resolve.ts"
import { ADDON_ENV } from "../settings.ts"
import { buildAddon, removeAddon } from "./addon.ts"
import { createHosts } from "./hosts.ts"

const ROOT = resolve(import.meta.dirname, "..", "..")
const TESTS = join(ROOT, "conformance", "tests")

// "<area>/<file>#<group>" to the open issue whose fix unblocks it in this runner alone;
// what the crate does not implement yet is conformance/pending.json, shared by every runner
const KNOWN_DIVERGENCES: Record<string, string> = {}
const PENDING: Record<string, string> = JSON.parse(
  readFileSync(join(ROOT, "conformance", "pending.json"), "utf8"),
)

type Expected = { error?: { code: string; message?: string } } & Record<string, unknown>
type Test = {
  description: string
  input?: string
  tree?: object
  operations?: unknown[]
  expected: Expected
}
type Group = {
  description: string
  pipeline: unknown[]
  positions?: boolean
  tests: Test[]
}

let runCase: (json: string) => string
let entry: typeof import("../index.ts")
let addonDir = ""
const OUTPUTS = ["html", "md", "js", "txt"]

beforeAll(async () => {
  addonDir = mkdtempSync(join(tmpdir(), "textlane-addon-"))
  const addonPath = buildAddon(addonDir)
  const require = createRequire(import.meta.url)
  const addon: { runCase: (json: string) => string } = require(
    resolveAddon({
      platform: process.platform,
      arch: process.arch,
      env: { [ADDON_ENV]: addonPath },
      resolvePackage: specifier => require.resolve(specifier),
    }),
  )
  runCase = addon.runCase
  process.env[ADDON_ENV] = addonPath
  entry = await import("../index.ts")
}, 600_000)

afterAll(() => {
  if (addonDir) removeAddon(addonDir)
})

describe("conformance", () => {
  const matchedEntries = new Set<string>()
  const failingEntries = new Set<string>()
  for (const file of listFiles(TESTS)) {
    const id = relative(TESTS, file)
      .replace(/\\/g, "/")
      .replace(/\.json$/, "")
    const groups: Group[] = JSON.parse(readFileSync(file, "utf8"))
    for (const group of groups) {
      const groupId = `${id}#${group.description}`
      if (groupId in KNOWN_DIVERGENCES) continue
      describe(groupId, () => {
        for (const test of group.tests) {
          const testId = `${groupId}#${test.description}`
          const entry = Object.keys(PENDING).find(key =>
            matchesEntry(key, id, groupId, testId),
          )
          if (entry) matchedEntries.add(entry)
          it(test.description, async () => {
            const parsed = isHosted(group)
              ? await runHosted(group, test)
              : runNative(group, test)
            const actual = group.positions === false ? stripPositions(parsed) : parsed
            if (!entry) return expectOutcome(actual, test.expected)
            try {
              expectOutcome(actual, test.expected)
            } catch {
              failingEntries.add(entry)
            }
          })
        }
      })
    }
  }

  // A pending entry whose cases all pass is stale and fails the run
  it("pending.json lists only failing cases", () => {
    const stale = [...matchedEntries].filter(entry => !failingEntries.has(entry))
    expect(stale).toEqual([])
  })
})

function runNative(group: Group, test: Test): Expected {
  return JSON.parse(
    runCase(
      JSON.stringify({
        pipeline: group.pipeline,
        input: test.input,
        tree: test.tree,
        operations: test.operations,
      }),
    ),
  )
}

// `host:` plugins stand for JavaScript ones, so their groups go through the public API with
// this runner's stand-ins, processing asynchronously as the Rust runner does
function isHosted(group: Group) {
  return group.pipeline.some(
    entry => Array.isArray(entry) && String(entry[0]).startsWith("host:"),
  )
}

async function runHosted(group: Group, test: Test): Promise<Expected> {
  const processor = entry.textlane()
  const hostFor = createHosts()
  for (const item of group.pipeline) {
    const [name, ...options]: unknown[] = Array.isArray(item) ? item : [item]
    const plugin = String(name)
    if (plugin.startsWith("host:")) processor.use(hostFor(plugin), ...options)
    else processor.use(plugin, ...options)
  }
  try {
    const file = await processor.process(test.input ?? "")
    const messages = file.messages.map(message => ({
      ruleId: message.ruleId,
      source: message.source,
      place: message.place,
    }))
    const outputs = Object.fromEntries(OUTPUTS.map(key => [key, file.value]))
    return { ...outputs, data: processor.data(), messages }
  } catch (error) {
    const code =
      error instanceof Error && "code" in error ? String(error.code) : "unknown"
    const message = error instanceof Error ? error.message : String(error)
    return { error: { code, message } }
  }
}

// A group with `positions: false` compares trees without them
function stripPositions<T>(value: T): T {
  return JSON.parse(
    JSON.stringify(value, (key, item) => (key === "position" ? undefined : item)),
  )
}

// An entry names a file, a group, one test as `<group>#<test>`, or an area as `<area>/*`
function matchesEntry(entry: string, fileId: string, groupId: string, testId: string) {
  if (entry.endsWith("*")) return fileId.startsWith(entry.slice(0, -1))
  return entry === fileId || entry === groupId || entry === testId
}

// A run may report several outcomes; the one `expected` names is compared, and an unexpected
// error always fails. An expected error matches on its code, its message only as a substring
function expectOutcome(actual: Expected, expected: Expected) {
  if (!expected.error) {
    expect(actual.error).toBeUndefined()
    for (const key of Object.keys(expected)) expect(actual[key]).toEqual(expected[key])
    return
  }
  expect(actual.error?.code).toBe(expected.error.code)
  if (expected.error.message)
    expect(actual.error?.message).toContain(expected.error.message)
}

function listFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return listFiles(path)
    return entry.name.endsWith(".json") ? [path] : []
  })
}
