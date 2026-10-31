import { createRequire } from "node:module"
import { resolveAddon } from "./actions/addon/resolve.ts"
import { detectLibc } from "./actions/libc/detect.ts"
import type { Addon } from "./models/addon.ts"
import { Processor } from "./models/processor.ts"

export type {
  Compatible,
  Message,
  MessageOptions,
  Node,
  Place,
  Point,
  VFile,
} from "./models/file.ts"
export type {
  Attacher,
  Pluggable,
  PluggableList,
  Parser,
  Plugin,
  Preset,
  Transformer,
} from "./models/plugin.ts"
export type { HostFile, HostMessage } from "./models/vfile.ts"
/** The unist node, position and data types every syntax tree shares. */
export type * as unist from "./generated/trees/unist.ts"
/** The Markdown tree `md.parse` makes, with the nodes of textlane's syntax extensions. */
export type * as mdast from "./generated/trees/mdast.ts"
/** The HTML tree `html.parse` and `md.toHtml` make, with MDX's and raw nodes. */
export type * as hast from "./generated/trees/hast.ts"
/** The natural-language tree `nl.parse` makes. */
export type * as nlcst from "./generated/trees/nlcst.ts"
/** The JavaScript tree MDX's nodes carry in `data.estree`, JSX included. */
export type * as estree from "./generated/trees/estree.ts"
export { Processor } from "./models/processor.ts"

const require = createRequire(import.meta.url)

const addon: Addon = require(
  resolveAddon({
    platform: process.platform,
    arch: process.arch,
    libc: detectLibc({
      platform: process.platform,
      report: () => process.report.getReport(),
    }),
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
