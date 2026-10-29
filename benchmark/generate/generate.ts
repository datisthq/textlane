import { existsSync } from "node:fs"
import { mkdir, rename, stat, writeFile } from "node:fs/promises"
import { join } from "node:path"
import type { Profile } from "../profiles/types.ts"
import { createRandom } from "./random.ts"
import { type Size, formatBytes, parseSize } from "./size.ts"

export type Dataset = {
  profile: string
  size: Size
  seed: number
  bytes: number
  path: string
}

/** Writes the profile's blocks until the document reaches its size; cached by name. */
export async function generateDataset(props: {
  profile: Profile
  size: Size
  directory: string
  seed?: number
}) {
  const seed = props.seed ?? 1
  const name = `${props.profile.name}-v${props.profile.version}-${props.size}-s${seed}.md`
  const path = join(props.directory, name)
  if (!existsSync(path)) {
    await mkdir(props.directory, { recursive: true })
    const text = createDocument({
      profile: props.profile,
      bytes: parseSize(props.size),
      seed,
    })
    // written aside first, so an interrupted run never leaves a short document cached
    await writeFile(`${path}.partial`, text)
    await rename(`${path}.partial`, path)
  }
  const dataset: Dataset = {
    profile: props.profile.name,
    size: props.size,
    seed,
    bytes: (await stat(path)).size,
    path,
  }
  return dataset
}

export function createDocument(props: { profile: Profile; bytes: number; seed: number }) {
  const random = createRandom(props.seed)
  const blocks: string[] = []
  let length = 0
  for (let index = 0; length < props.bytes; index++) {
    const block = props.profile.createBlock(random, index)
    blocks.push(block)
    length += Buffer.byteLength(block)
  }
  return blocks.join("")
}

export function describeDataset(dataset: Dataset) {
  return `${dataset.profile} ${dataset.size} (${formatBytes(dataset.bytes)})`
}
