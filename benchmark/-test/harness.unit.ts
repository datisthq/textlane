import { describe, expect, it } from "vite-plus/test"
import { readAdapterOutput, summarize } from "../harness/measure.ts"
import { renderReport } from "../report/render.ts"
import { TOOLS, selectTools } from "../tools/index.ts"

describe("readAdapterOutput", () => {
  it("reads the last line an adapter prints", () => {
    const stdout = 'warming up\n{"elapsedMs": 12.5, "outputBytes": 300}\n'
    expect(readAdapterOutput(stdout)).toEqual({ elapsedMs: 12.5, outputBytes: 300 })
  })

  it("rejects output without both numbers", () => {
    expect(readAdapterOutput('{"elapsedMs": 1}')).toBeUndefined()
    expect(readAdapterOutput("Traceback")).toBeUndefined()
  })
})

describe("summarize", () => {
  it("takes the median of an even series as the middle pair's mean", () => {
    expect(summarize([4, 1, 3, 2])).toEqual({ median: 2.5, min: 1, max: 4 })
  })
})

describe("selectTools", () => {
  it("names unknown tools", () => {
    expect(selectTools()).toBe(TOOLS)
    expect(() => selectTools(["nope"])).toThrow(/Unknown tools nope/)
  })

  it("gives every tool a unique name", () => {
    const names = TOOLS.map(tool => tool.name)
    expect(new Set(names).size).toBe(names.length)
  })
})

describe("renderReport", () => {
  it("escapes what the results carry", () => {
    const html = renderReport({
      measuredAt: "2026-10-29T00:00:00.000Z",
      machine: { cpu: "<cpu>", cpus: 8, memory: "16GB", os: "linux" },
      results: [
        {
          case: "prose",
          tool: "comrak",
          title: "comrak",
          language: "rust",
          version: "1.0.0",
          profile: "prose",
          size: "10KB",
          bytes: 10240,
          repeat: 820,
          runs: 1,
          documentMs: { median: 0.1, min: 0.1, max: 0.1 },
          throughputMbs: 97.7,
          outcome: "ok",
          outputBytes: 12000,
          caveats: ["a <b> caveat"],
        },
      ],
    })
    expect(html).toContain("&lt;cpu&gt;")
    expect(html).toContain("a &lt;b&gt; caveat")
    expect(html).toContain("100 µs")
  })
})
