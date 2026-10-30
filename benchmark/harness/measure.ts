import { dirname } from "node:path"
import type { Dataset } from "../generate/generate.ts"
import type { Language, Tool } from "../tools/types.ts"
import type { Flavor } from "./case.ts"
import { type Outcome, runCommand } from "./run.ts"

export type Result = {
  case: string
  tool: string
  title: string
  language: Language
  isTextlane?: boolean
  version: string
  profile: string
  size: string
  bytes: number
  /** How many times each run converts the document, so small ones outlast the clock. */
  repeat: number
  runs: number
  /** Time to convert the document once, from the adapter's clock after its imports. */
  documentMs?: { median: number; min: number; max: number }
  /** Input megabytes converted per second, at the median. */
  throughputMbs?: number
  /** Peak anonymous memory of the process tree, sampled. */
  peakMemoryMb?: number
  outcome: Outcome
  /** How the failed run ended, for example `code 1` or `signal 9`. */
  exit?: string
  /** Bytes of HTML the last conversion produced; far off the others means a different render. */
  outputBytes?: number
  caveats: string[]
  stderr?: string
}

/** Converted per run at least, so a 10KB document is timed over hundreds of conversions. */
export const MIN_RUN_BYTES = 8 * 1024 * 1024

// One tool's runs so far; the first failed run ends it
type Series = {
  tool: Tool
  timings: number[]
  memories: number[]
  outputBytes?: number
  outcome: Outcome
  exit?: string
  stderr?: string
}

/**
 * Runs every tool on one dataset `warmup + runs` times, one run of each tool per round, so
 * load that comes and goes while they run lands on all of them alike.
 */
export async function measureTools(props: {
  caseName: string
  tools: Tool[]
  dataset: Dataset
  flavor: Flavor
  runs: number
  warmup: number
  timeoutMs: number
  memoryMax?: string
}) {
  const repeat = Math.max(1, Math.ceil(MIN_RUN_BYTES / props.dataset.bytes))
  const series: Series[] = props.tools.map(tool => ({
    tool,
    timings: [],
    memories: [],
    outcome: "ok",
  }))

  for (let index = 0; index < props.warmup + props.runs; index++) {
    for (const item of series) {
      if (item.outcome !== "ok") continue
      const command = await item.tool.createCommand(props.dataset, repeat, props.flavor)
      const run = await runCommand(command, {
        cwd: dirname(props.dataset.path),
        timeoutMs: props.timeoutMs,
        memoryMax: props.memoryMax,
      })
      const output = run.outcome === "ok" ? readAdapterOutput(run.stdout) : undefined
      if (run.outcome !== "ok" || !output) {
        item.outcome = run.outcome === "ok" ? "crash" : run.outcome
        item.exit = run.exit
        item.stderr = run.stderr.slice(-2000)
        continue
      }
      if (index < props.warmup) continue
      item.timings.push(output.elapsedMs / repeat)
      if (run.peakMemoryMb !== undefined) item.memories.push(run.peakMemoryMb)
      item.outputBytes = output.outputBytes
    }
  }

  const results: Result[] = []
  for (const item of series) {
    const documentMs = item.timings.length ? summarize(item.timings) : undefined
    results.push({
      case: props.caseName,
      tool: item.tool.name,
      title: item.tool.title,
      language: item.tool.language,
      isTextlane: item.tool.isTextlane,
      version: await item.tool.getVersion().catch(() => "unknown"),
      profile: props.dataset.profile,
      size: props.dataset.size,
      bytes: props.dataset.bytes,
      repeat,
      runs: item.timings.length,
      documentMs,
      throughputMbs: documentMs
        ? props.dataset.bytes / 1024 / 1024 / (documentMs.median / 1000)
        : undefined,
      peakMemoryMb: item.memories.length ? Math.max(...item.memories) : undefined,
      outcome: item.outcome,
      exit: item.exit,
      outputBytes: item.outputBytes,
      caveats: item.tool.caveats,
      stderr: item.stderr,
    })
  }
  return results
}

/** Reads the `{"elapsedMs", "outputBytes"}` line every adapter prints last. */
export function readAdapterOutput(stdout: string) {
  try {
    const output: unknown = JSON.parse(stdout.trim().split("\n").at(-1) ?? "")
    if (
      typeof output === "object" &&
      output !== null &&
      "elapsedMs" in output &&
      "outputBytes" in output &&
      typeof output.elapsedMs === "number" &&
      typeof output.outputBytes === "number"
    ) {
      return { elapsedMs: output.elapsedMs, outputBytes: output.outputBytes }
    }
  } catch {}
  return undefined
}

export function summarize(values: number[]) {
  const sorted = values.toSorted((left, right) => left - right)
  const middle = Math.floor(sorted.length / 2)
  const median =
    sorted.length % 2
      ? (sorted[middle] ?? 0)
      : ((sorted[middle - 1] ?? 0) + (sorted[middle] ?? 0)) / 2
  return { median, min: sorted[0] ?? 0, max: sorted.at(-1) ?? 0 }
}
