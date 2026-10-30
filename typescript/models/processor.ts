import { isAttacher } from "../actions/attacher/check.ts"
import { createFile } from "../actions/file/create.ts"
import { isPlainObject, mergeOptions } from "../actions/options/merge.ts"
import { type HostEntry, mapPluggable } from "../actions/pluggable/map.ts"
import { finishRun } from "../actions/run/finish.ts"
import { isTransformer } from "../actions/transformer/check.ts"
import type { NativePipeline } from "./addon.ts"
import type { Compatible, Node, VFile } from "./file.ts"
import type { Host } from "./host.ts"
import type { Attacher, Pluggable, PluggableList, Preset } from "./plugin.ts"

/**
 * unified's processor over the native pipeline; every call is the crate's, except that
 * JavaScript plugins run here when a run pauses before them.
 */
export class Processor {
  #native: NativePipeline
  #ids = new Map<Attacher, number>()
  #hosts = new Map<number, Host>()
  #attached = false

  constructor(native: NativePipeline) {
    this.#native = native
  }

  /**
   * Add a plugin, a list of them or a preset; a plugin used again is reconfigured. A
   * JavaScript plugin is an attacher, called with the processor and its options at freeze.
   */
  use(
    value: Pluggable | PluggableList | Preset | null | undefined,
    ...options: unknown[]
  ) {
    if (isAttacher(value)) {
      this.#native.use([this.#register(value, options)])
      return this
    }
    const register = (attacher: Attacher, parameters: unknown[]) =>
      this.#register(attacher, parameters)
    this.#native.use(mapPluggable(value ?? null, register), options[0])
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
    this.#attach()
    this.#native.freeze()
    return this
  }

  /** Parse text into a tree. */
  parse(text: string) {
    this.#attach()
    return this.#native.parse(text)
  }

  /**
   * Run the transforms, finishing asynchronous stages; `file`, the text `tree` was parsed
   * from, lets bridges read it and keeps positions in the result.
   */
  async run(tree: Node, file?: Compatible) {
    if (!this.#attach()) return this.#native.run(tree, readValue(file))
    const run = this.#native.startRun(tree, readValue(file), false)
    await finishRun({
      run,
      hosts: this.#hosts,
      file: createFile(readValue(file)),
      sync: false,
      call: "run",
    })
    return run.result()
  }

  /** Run the transforms; fails with `async-in-sync` on an asynchronous stage. */
  runSync(tree: Node, file?: Compatible) {
    if (!this.#attach()) return this.#native.runSync(tree, readValue(file))
    const run = this.#native.startRun(tree, readValue(file), true)
    void finishRun({
      run,
      hosts: this.#hosts,
      file: createFile(readValue(file)),
      sync: true,
      call: "run",
    })
    return run.result()
  }

  /** Compile a tree, running no transform. */
  stringify(tree: Node) {
    this.#attach()
    return this.#native.stringify(tree)
  }

  /** Parse, run and compile, finishing asynchronous stages. */
  async process(text: string): Promise<VFile> {
    if (!this.#attach()) return this.#native.process(text)
    const run = this.#native.startProcess(text, false)
    const file = createFile(run.value() ?? text)
    await finishRun({ run, hosts: this.#hosts, file, sync: false, call: "process" })
    return { ...run.result(), data: file.data }
  }

  /** Parse, run and compile; fails with `async-in-sync` on an asynchronous stage. */
  processSync(text: string) {
    if (!this.#attach()) return this.#native.processSync(text)
    const run = this.#native.startProcess(text, true)
    const file = createFile(run.value() ?? text)
    void finishRun({ run, hosts: this.#hosts, file, sync: true, call: "process" })
    return { ...run.result(), data: file.data }
  }

  // unified's addPlugin: the same attacher is one plugin, its object options merged
  #register(attacher: Attacher, parameters: unknown[]): HostEntry {
    let id = this.#ids.get(attacher)
    if (id === undefined) {
      id = this.#ids.size
      this.#ids.set(attacher, id)
      this.#hosts.set(id, { attacher, parameters })
      return [{ host: id }, parameters[0] !== false]
    }
    const host = this.#hosts.get(id)
    if (!host || parameters.length === 0) return [{ host: id }]
    const [primary, ...rest] = parameters
    const merged =
      isPlainObject(host.parameters[0]) && isPlainObject(primary)
        ? mergeOptions(host.parameters[0], primary)
        : primary
    host.parameters = [merged, ...rest]
    return [{ host: id }, merged !== false]
  }

  // Attachers run once, before the native pipeline freezes, so they can still set data;
  // whether any JavaScript plugin is on decides if runs pause for them
  #attach() {
    if (!this.#attached) {
      this.#attached = true
      for (const id of this.#native.hosts()) {
        const host = this.#hosts.get(id)
        if (!host) continue
        const [primary, ...rest] = host.parameters
        const parameters = primary === true ? [undefined, ...rest] : host.parameters
        const transformer: unknown = Reflect.apply(host.attacher, this, parameters)
        if (isTransformer(transformer)) host.transformer = transformer
      }
    }
    return this.#native.hosts().length > 0
  }
}

function readValue(file: Compatible | undefined) {
  return typeof file === "string" ? file : file?.value
}
