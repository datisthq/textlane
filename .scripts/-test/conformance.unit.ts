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
})

function listFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return listFiles(path)
    return entry.name.endsWith(".json") ? [path] : []
  })
}
