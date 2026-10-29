import type { Node, VFile } from "./file.ts"

/** The crate's pipeline as the addon exposes it: values cross as JSON. */
export interface NativePipeline {
  use(value: unknown, options?: unknown): void
  data(key?: string): unknown
  setData(key: string, value: unknown): void
  replaceData(data: Record<string, unknown>): void
  freeze(): void
  parse(text: string): Node
  run(tree: Node, value?: string): Node
  runSync(tree: Node, value?: string): Node
  stringify(tree: Node): string
  process(text: string): VFile
  processSync(text: string): VFile
}

/** What the napi addon exports. */
export interface Addon {
  version(): string
  pluginNames(): string[]
  Pipeline: new () => NativePipeline
}
