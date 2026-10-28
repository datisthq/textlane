import { readdirSync, readFileSync } from "node:fs"
import { join, relative, resolve } from "node:path"
import { Ajv2020 } from "ajv/dist/2020.js"
import { describe, expect, it } from "vite-plus/test"

const SUITE = resolve(import.meta.dirname, "..", "..", "conformance")
const TESTS = join(SUITE, "tests")

describe("the conformance suite", () => {
  const schema = JSON.parse(readFileSync(join(SUITE, "test-schema.json"), "utf8"))
  const validate = new Ajv2020({ allErrors: true, validateFormats: false }).compile(
    schema,
  )
  const files = listFiles(TESTS)

  it("has test files", () => {
    expect(files.length).toBeGreaterThan(0)
  })

  it.each(files.map(file => relative(TESTS, file)))(
    "%s matches test-schema.json",
    name => {
      const groups = JSON.parse(readFileSync(join(TESTS, name), "utf8"))
      validate(groups)
      expect(validate.errors ?? []).toEqual([])
      const descriptions = groups.map(
        (group: { description: string }) => group.description,
      )
      expect(new Set(descriptions).size).toBe(descriptions.length)
    },
  )

  it("lists only existing files, groups or areas in pending.json", () => {
    const pending: Record<string, string> = JSON.parse(
      readFileSync(join(SUITE, "pending.json"), "utf8"),
    )
    const ids = files.flatMap(file => {
      const fileId = relative(TESTS, file)
        .replace(/\\/g, "/")
        .replace(/\.json$/, "")
      const groups: { description: string }[] = JSON.parse(readFileSync(file, "utf8"))
      return [fileId, ...groups.map(group => `${fileId}#${group.description}`)]
    })
    const unknown = Object.keys(pending).filter(entry =>
      entry.endsWith("*")
        ? !ids.some(id => id.startsWith(entry.slice(0, -1)))
        : !ids.includes(entry),
    )
    expect(unknown).toEqual([])
    expect(Object.values(pending).every(reason => reason.length > 0)).toBe(true)
  })
})

function listFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return listFiles(path)
    return entry.name.endsWith(".json") ? [path] : []
  })
}
