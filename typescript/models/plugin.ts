import type { Node } from "./file.ts"
import type { HostFile } from "./vfile.ts"

/** A native plugin as a pipeline names it, with its options when given. */
export interface Plugin<Name extends string = string, Options = unknown> {
  name: Name
  options?: Options
}

/** Plugins and the settings they share, as unified's presets. */
export interface Preset {
  plugins?: PluggableList
  settings?: Record<string, unknown>
}

/**
 * A JavaScript plugin, as unified's attachers: called with the processor as `this` and its
 * options when the processor freezes, it returns the transformer the pipeline runs.
 */
export type Attacher = (this: never, ...parameters: never[]) => unknown

/**
 * What an attacher returns: it changes the tree in place or returns a new one, directly,
 * through a promise or through `next`.
 */
export type Transformer = (
  tree: Node,
  file: HostFile,
  next: (error?: unknown, tree?: Node) => void,
) => unknown

/** A plugin, by descriptor, name or attacher, with options, or a preset. */
export type Pluggable =
  | Plugin
  | string
  | Attacher
  | [Plugin | string | Attacher, unknown?]
  | Preset

/** Several pluggables, used in order. */
export type PluggableList = Pluggable[]
