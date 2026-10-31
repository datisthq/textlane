import { mkdtempSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import rehypeAutolinkHeadings from "rehype-autolink-headings"
import { afterAll, beforeAll, describe, expect, it } from "vite-plus/test"
import { slides } from "../../.livemark/components/hero-slides.ts"
import type { Stage } from "../../.livemark/components/hero-slides.ts"
import { buildAddon, removeAddon } from "../../typescript/-test/addon.ts"
import type { HostMessage } from "../../typescript/index.ts"
import { ADDON_ENV } from "../../typescript/settings.ts"

const HOSTS: Record<string, unknown> = { rehypeAutolinkHeadings }

let entry: typeof import("../../typescript/index.ts")
let addonDir = ""

beforeAll(async () => {
  addonDir = mkdtempSync(join(tmpdir(), "textlane-addon-"))
  process.env[ADDON_ENV] = buildAddon(addonDir)
  entry = await import("../../typescript/index.ts")
}, 600_000)

afterAll(() => {
  if (addonDir) removeAddon(addonDir)
})

describe("the landing page's hero slides", () => {
  it.each(slides.map(slide => [slide.id, slide] as const))(
    "%s shows what its pipeline outputs",
    async (_, slide) => {
      const plugins = slide.stages.map(stage => readPlugin(stage))
      const file = await entry.textlane().use(plugins).process(slide.document)
      const output =
        slide.output.lang === "report"
          ? formatReport(slide.input.label, file.messages)
          : String(file.value)
      expect(output).toBe(slide.output.text)
    },
  )

  it("shows the processed document in a code input", () => {
    for (const slide of slides) {
      if (slide.code) expect(slide.code).toContain(JSON.stringify(slide.document))
    }
  })
})

function readPlugin(stage: Stage) {
  if (!stage.host) return { name: stage.name, options: stage.options }
  const plugin = HOSTS[stage.name]
  if (typeof plugin !== "function") throw new Error(`no host plugin ${stage.name}`)
  return plugin
}

// As vfile-reporter lays out warnings, sorted by place, the place column padded
function formatReport(label: string, messages: HostMessage[]) {
  const rows = messages
    .toSorted((a, b) => (a.place?.start.column ?? 0) - (b.place?.start.column ?? 0))
    .map(message => {
      const place = message.place
        ? `${message.place.start.line}:${message.place.start.column}-${message.place.end.line}:${message.place.end.column}`
        : ""
      return [place, message.reason, message.ruleId ?? "", message.source ?? ""] as const
    })
  const width = Math.max(...rows.map(row => row[0].length))
  const lines = rows.map(
    row => `  ${row[0].padEnd(width)}  warning  ${row[1]}  ${row[2]}  ${row[3]}`,
  )
  return [label, ...lines, "", `⚠ ${rows.length} warnings`].join("\n")
}
