import { describe, expect, it } from "vite-plus/test"
import { detectLibc } from "./detect.ts"

describe("detectLibc", () => {
  it("finds glibc when the report names its version", () => {
    const report = () => ({ header: { glibcVersionRuntime: "2.39" } })
    expect(detectLibc({ platform: "linux", report })).toBe("glibc")
  })

  it("finds musl when the report names no glibc", () => {
    const report = () => ({ header: {} })
    expect(detectLibc({ platform: "linux", report })).toBe("musl")
  })

  it("keeps glibc when the report has no header", () => {
    expect(detectLibc({ platform: "linux", report: () => ({}) })).toBe("glibc")
  })

  it("asks for no report off Linux", () => {
    const report = () => {
      throw new Error("not called")
    }
    expect(detectLibc({ platform: "darwin", report })).toBeUndefined()
  })
})
