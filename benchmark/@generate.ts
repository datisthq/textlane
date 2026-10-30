import { join } from "node:path"
import { parseArgs } from "node:util"
import { describeDataset, generateDataset } from "./generate/generate.ts"
import { getProfile } from "./profiles/index.ts"

const { values } = parseArgs({
  options: {
    profile: { type: "string", default: "readme" },
    size: { type: "string", default: "1MB" },
    seed: { type: "string", default: "1" },
  },
})

const started = performance.now()
const dataset = await generateDataset({
  profile: getProfile(values.profile),
  size: values.size,
  directory: join(import.meta.dirname, ".cache", "data"),
  seed: Number(values.seed),
})

const seconds = ((performance.now() - started) / 1000).toFixed(1)
console.log(`${describeDataset(dataset)} in ${seconds}s`)
console.log(dataset.path)
