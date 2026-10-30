import type { Message, Node, VFile } from "./file.ts"

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
  hosts(): number[]
  startRun(tree: Node, value: string | undefined, sync: boolean): NativeRun<Node>
  startProcess(text: string, sync: boolean): NativeRun<VFile>
}

/** A run paused before host plugins until `resume`, then finished with `result`. */
export interface NativeRun<Result> {
  hosts(): number[]
  tree(): Node
  value(): string | null
  resume(tree: Node, messages: Message[]): void
  result(): Result
}

/** What the napi addon exports. */
export interface Addon {
  version(): string
  pluginNames(): string[]
  retainedMemory(): number
  setRetainedMemory(bytes: number): void
  Pipeline: new () => NativePipeline
}
