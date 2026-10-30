import { readFile, readdir } from "node:fs/promises"
import { basename, join } from "node:path"
import type { Size } from "../generate/size.ts"

/** The Markdown an adapter parses: CommonMark, or GFM with every extension a library has. */
export type Flavor = "commonmark" | "gfm"

export type Case = {
  name: string
  description: string
  profile: string
  /** `commonmark` when left out. */
  flavor?: Flavor
  sizes: Size[]
  /** The only tools measured at a size, for sizes too slow for every rival. */
  tools?: Record<Size, string[]>
  seed?: number
}

export const CASES = join(import.meta.dirname, "..", "cases")

export async function loadCases(names?: string[]) {
  const files = (await readdir(CASES)).filter(file => file.endsWith(".json")).toSorted()
  const cases: Case[] = []
  for (const file of files) {
    const name = basename(file, ".json")
    if (names?.length && !names.includes(name)) continue
    cases.push({ name, ...JSON.parse(await readFile(join(CASES, file), "utf8")) })
  }
  if (names?.length && cases.length !== names.length) {
    throw new Error(`Unknown case in ${names.join(", ")}; known: ${files.join(", ")}`)
  }
  return cases
}
