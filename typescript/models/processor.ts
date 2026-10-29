import type { NativePipeline } from "./addon.ts"
import type { Compatible, Node, VFile } from "./file.ts"
import type { Pluggable, PluggableList, Preset } from "./plugin.ts"

/** unified's processor over the native pipeline; every call is the crate's. */
export class Processor {
  #native: NativePipeline

  constructor(native: NativePipeline) {
    this.#native = native
  }

  /** Add a plugin, a list of them or a preset; a plugin used again is reconfigured. */
  use(value: Pluggable | PluggableList | Preset | null | undefined, options?: unknown) {
    this.#native.use(value ?? null, options)
    return this
  }

  /** All data, or the value under `key`. */
  data(): Record<string, unknown>
  data(key: string): unknown
  /** Set the value under `key`, or replace all data. */
  data(key: string, value: unknown): this
  data(dataset: Record<string, unknown>): this
  data(key?: string | Record<string, unknown>, ...value: unknown[]) {
    if (typeof key === "object") {
      this.#native.replaceData(key)
      return this
    }
    if (key !== undefined && value.length > 0) {
      this.#native.setData(key, value[0])
      return this
    }
    return this.#native.data(key)
  }

  /** Attach every plugin; later `use` and `data` setters fail. */
  freeze() {
    this.#native.freeze()
    return this
  }

  /** Parse text into a tree. */
  parse(text: string) {
    return this.#native.parse(text)
  }

  /**
   * Run the transforms, finishing asynchronous stages; `file`, the text `tree` was parsed
   * from, lets bridges read it and keeps positions in the result.
   */
  async run(tree: Node, file?: Compatible) {
    return this.#native.run(tree, readValue(file))
  }

  /** Run the transforms; fails with `async-in-sync` on an asynchronous stage. */
  runSync(tree: Node, file?: Compatible) {
    return this.#native.runSync(tree, readValue(file))
  }

  /** Compile a tree, running no transform. */
  stringify(tree: Node) {
    return this.#native.stringify(tree)
  }

  /** Parse, run and compile, finishing asynchronous stages. */
  async process(text: string): Promise<VFile> {
    return this.#native.process(text)
  }

  /** Parse, run and compile; fails with `async-in-sync` on an asynchronous stage. */
  processSync(text: string) {
    return this.#native.processSync(text)
  }
}

function readValue(file: Compatible | undefined) {
  return typeof file === "string" ? file : file?.value
}
