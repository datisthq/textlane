import { createRequire } from "node:module"
import { resolveAddon } from "./actions/addon/resolve.ts"
import type { Addon } from "./models/addon.ts"
import { Processor } from "./models/processor.ts"

export type { Compatible, Message, Node, Point, VFile } from "./models/file.ts"
export type { Pluggable, PluggableList, Plugin, Preset } from "./models/plugin.ts"
export { Processor } from "./models/processor.ts"

const require = createRequire(import.meta.url)

const addon: Addon = require(
  resolveAddon({
    platform: process.platform,
    arch: process.arch,
    env: process.env,
    resolvePackage: specifier => require.resolve(specifier),
  }),
)

/** A new processor: add plugins with `use`, then `process` text. */
export function textlane() {
  return new Processor(new addon.Pipeline())
}

/** The package version, as the native crate reports it. */
export const version = addon.version

/** The name of every native plugin, as a pipeline names it. */
export const pluginNames = addon.pluginNames

/** The most memory, in bytes, each thread keeps after a document to reuse for the next. */
export const retainedMemory = addon.retainedMemory

/**
 * Cap the memory, in bytes, each thread keeps after a document; `0` keeps none. The
 * default is 256 MB.
 */
export const setRetainedMemory = addon.setRetainedMemory
