import { readFileSync } from "node:fs"

/** Whether the third argument, the case's flavor, asks for GFM. */
export const isGfm = process.argv[4] === "gfm"

/**
 * Converts `<input> <repeat>` times and prints `{elapsedMs, outputBytes}`; imports and one
 * warm-up conversion are untimed.
 */
export function runAdapter(convert: (markdown: string) => string) {
  const [path = "", repeat = "1"] = process.argv.slice(2)
  const markdown = readFileSync(path, "utf8")
  let html = convert(markdown)
  const started = performance.now()
  for (let index = 0; index < Number(repeat); index++) html = convert(markdown)
  const elapsedMs = performance.now() - started
  console.log(JSON.stringify({ elapsedMs, outputBytes: Buffer.byteLength(html) }))
}
