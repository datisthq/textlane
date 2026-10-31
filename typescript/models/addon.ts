import type { Message, VFile } from "./file.ts"

/** The crate's pipeline as the addon exposes it: values cross as JSON, trees as JSON text. */
export interface NativePipeline {
  use(value: unknown, options?: unknown): void
  data(key?: string): unknown
  setData(key: string, value: unknown): void
  replaceData(data: Record<string, unknown>): void
  freeze(): void
  parse(text: string): string
  run(tree: string, value?: string): string
  runSync(tree: string, value?: string): string
  stringify(tree: string): string
  process(text: string, path?: string): Promise<ProcessOutcome>
  processSync(text: string, path?: string): VFile
  hosts(): number[]
  startRun(tree: string, value: string | undefined, sync: boolean): NativeRun
  startProcess(text: string, sync: boolean, path?: string): NativeRun
}

/** What an async `process` settles with: the file, or the error the crate raised. */
export interface ProcessOutcome {
  file?: VFile
  error?: { code: string; message: string }
}

/** A run paused before host plugins until `resume`, then finished with `result`, as JSON
 * text: the tree for a run, the file for a process. */
export interface NativeRun {
  hosts(): number[]
  tree(): string
  value(): string | null
  messages(): Message[]
  resume(tree: string, messages: Message[]): void
  result(): string
}

/** What the napi addon exports. */
export interface Addon {
  version(): string
  pluginNames(): string[]
  retainedMemory(): number
  setRetainedMemory(bytes: number): void
  Pipeline: new () => NativePipeline
}
