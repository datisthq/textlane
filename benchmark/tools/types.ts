import type { Dataset } from "../generate/generate.ts"
import type { Flavor } from "../harness/case.ts"

export type Language = "rust" | "typescript" | "python"

export type Tool = {
  name: string
  title: string
  language: Language
  /** One of textlane's own adapters, set apart from the rivals in the report. */
  isTextlane?: boolean
  /** What the tool renders or measures differently from textlane, shown in the report. */
  caveats: string[]
  getVersion: () => Promise<string>
  /** The command converting the dataset `repeat` times as `flavor`, run from its directory. */
  createCommand: (dataset: Dataset, repeat: number, flavor: Flavor) => Promise<Command>
}

export type Command = {
  file: string
  args: string[]
  /** Exit codes of a completed run; `[0]` by default. */
  okExitCodes?: number[]
  /** Extra environment, such as where the textlane addon lives. */
  env?: Record<string, string>
}
