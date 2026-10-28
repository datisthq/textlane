import { execFileSync } from "node:child_process"
import { copyFileSync, mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs"
import { createRequire } from "node:module"
import { tmpdir } from "node:os"
import { join, relative, resolve } from "node:path"
import { afterAll, beforeAll, describe, expect, it } from "vite-plus/test"
import { resolveAddon } from "../actions/addon/resolve.ts"
import { ADDON_ENV } from "../settings.ts"

const ROOT = resolve(import.meta.dirname, "..", "..")
const TESTS = join(ROOT, "conformance", "tests")

// "<area>/<file>#<group>" to the open issue whose fix unblocks it in this runner alone;
// what the crate does not implement yet is conformance/pending.json, shared by every runner
const KNOWN_DIVERGENCES: Record<string, string> = {}
const PENDING: Record<string, string> = JSON.parse(
  readFileSync(join(ROOT, "conformance", "pending.json"), "utf8"),
)

const LIBRARY_NAMES: Record<string, string> = {
  darwin: "libtextlane.dylib",
  linux: "libtextlane.so",
  win32: "textlane.dll",
}

type Expected = { error?: { code: string; message?: string } } & Record<string, unknown>
type Test = { description: string; input?: string; tree?: object; expected: Expected }
type Group = { description: string; pipeline: unknown[]; tests: Test[] }

let runCase: (json: string) => string
let addonDir = ""

beforeAll(() => {
  const rust = join(ROOT, "rust")
  execFileSync("cargo", ["build", "-p", "textlane", "--features", "node,testing"], {
    cwd: rust,
    stdio: "inherit",
  })
  const library = LIBRARY_NAMES[process.platform]
  if (!library) throw new Error(`no cargo library name known for ${process.platform}`)
  addonDir = mkdtempSync(join(tmpdir(), "textlane-addon-"))
  const addonPath = join(addonDir, "textlane.node")
  copyFileSync(join(rust, "target", "debug", library), addonPath)
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
}, 600_000)

afterAll(() => {
  if (addonDir) rmSync(addonDir, { recursive: true, force: true })
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
      const entry = Object.keys(PENDING).find(key => matchesEntry(key, id, groupId))
      if (entry) matchedEntries.add(entry)
      describe(groupId, () => {
        for (const test of group.tests) {
          it(test.description, () => {
            const actual: Expected = JSON.parse(
              runCase(
                JSON.stringify({
                  pipeline: group.pipeline,
                  input: test.input,
                  tree: test.tree,
                }),
              ),
            )
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

// An entry names a file, a group, or a whole area as `<area>/*`
function matchesEntry(entry: string, fileId: string, groupId: string) {
  if (entry.endsWith("*")) return fileId.startsWith(entry.slice(0, -1))
  return entry === fileId || entry === groupId
}

// An expected error matches on its code, and on its message only as a substring
function expectOutcome(actual: Expected, expected: Expected) {
  if (!expected.error) return expect(actual).toEqual(expected)
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
