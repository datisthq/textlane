import type { Random } from "../generate/random.ts"

export type Profile = {
  name: string
  description: string
  /** Bumped whenever the output changes, so cached documents are regenerated. */
  version: number
  /** Writes the next block of Markdown, ending in a blank line. */
  createBlock: (random: Random, index: number) => string
}
