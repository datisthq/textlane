import { dirname } from "node:path"
import type { Dataset } from "../generate/generate.ts"
import type { Language, Tool } from "../tools/types.ts"
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

/** Runs one tool on one dataset `warmup + runs` times; the first failed run ends the series. */
export async function measureTool(props: {
  caseName: string
  tool: Tool
  dataset: Dataset
  runs: number
  warmup: number
  timeoutMs: number
  memoryMax?: string
}) {
  const repeat = Math.max(1, Math.ceil(MIN_RUN_BYTES / props.dataset.bytes))
  const timings: number[] = []
  const memories: number[] = []
  let outputBytes: number | undefined
  let outcome: Outcome = "ok"
  let exit: string | undefined
  let stderr: string | undefined

  for (let index = 0; index < props.warmup + props.runs; index++) {
    const command = await props.tool.createCommand(props.dataset, repeat)
    const run = await runCommand(command, {
      cwd: dirname(props.dataset.path),
      timeoutMs: props.timeoutMs,
      memoryMax: props.memoryMax,
    })
    const output = run.outcome === "ok" ? readAdapterOutput(run.stdout) : undefined
    if (run.outcome === "ok" && !output) run.outcome = "crash"
    if (run.outcome !== "ok" || !output) {
      outcome = run.outcome
      exit = run.exit
      stderr = run.stderr.slice(-2000)
      break
    }
    if (index < props.warmup) continue
    timings.push(output.elapsedMs / repeat)
    if (run.peakMemoryMb !== undefined) memories.push(run.peakMemoryMb)
    outputBytes = output.outputBytes
  }

  const documentMs = timings.length ? summarize(timings) : undefined
  const result: Result = {
    case: props.caseName,
    tool: props.tool.name,
    title: props.tool.title,
    language: props.tool.language,
    isTextlane: props.tool.isTextlane,
    version: await props.tool.getVersion().catch(() => "unknown"),
    profile: props.dataset.profile,
    size: props.dataset.size,
    bytes: props.dataset.bytes,
    repeat,
    runs: timings.length,
    documentMs,
    throughputMbs: documentMs
      ? props.dataset.bytes / 1024 / 1024 / (documentMs.median / 1000)
      : undefined,
    peakMemoryMb: memories.length ? Math.max(...memories) : undefined,
    outcome,
    exit,
    outputBytes,
    caveats: props.tool.caveats,
    stderr,
  }
  return result
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
