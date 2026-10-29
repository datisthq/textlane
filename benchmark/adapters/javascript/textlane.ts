import { createRequire } from "node:module"
import { runAdapter } from "./common.ts"

const PIPELINE = [
  ["md.parse"],
  ["md.toHtml", { allowDangerousHtml: true }],
  ["html.stringify", { allowDangerousHtml: true }],
]

// the bindings expose no pipeline yet, so this drives the `testing` addon's case runner
const addon: { runCase: (json: string) => string } = createRequire(import.meta.url)(
  process.env.TEXTLANE_ADDON ?? "",
)

runAdapter(markdown => {
  const result: { html?: string; error?: { message: string } } = JSON.parse(
    addon.runCase(JSON.stringify({ pipeline: PIPELINE, input: markdown })),
  )
  if (result.html === undefined) throw new Error(result.error?.message ?? "no html")
  return result.html
})
