import { isAttacher } from "../actions/attacher/check.ts"
import { isPlainObject, mergeOptions } from "../actions/options/merge.ts"
import { type HostEntry, mapPluggable } from "../actions/pluggable/map.ts"
import { finishRun } from "../actions/run/finish.ts"
import { isTransformer } from "../actions/transformer/check.ts"
import type { Node as UnistNode } from "unist"
import type { NativePipeline } from "./addon.ts"
import type { Compatible, Node, VFile } from "./file.ts"
import type { Host } from "./host.ts"
import type { Attacher, Parser, Pluggable, PluggableList, Preset } from "./plugin.ts"
import { HostFile } from "./vfile.ts"

/**
 * unified's processor over the native pipeline; every call is the crate's, except that
 * JavaScript plugins run here when a run pauses before them.
 */
export class Processor<Tree extends UnistNode = Node> {
  #native: NativePipeline
  #ids = new Map<Attacher, number>()
  #hosts = new Map<number, Host>()
  #attached = false
  #frozen = false

  constructor(native: NativePipeline) {
    this.#native = native
  }

  /**
   * Add a plugin, a list of them or a preset; a plugin used again is reconfigured. A
   * JavaScript plugin is an attacher, called with the processor and its options at freeze.
   * A parser, such as `md.parse()`, types the tree `parse` returns.
   */
  use<Value extends Pluggable | PluggableList | Preset | null | undefined>(
    value: Value,
    ...options: unknown[]
  ): Processor<ParsedTree<Value, Tree>>
  // a parser retypes this same processor: the tree it parses lives in the type alone
  use(
    value: Pluggable | PluggableList | Preset | null | undefined,
    ...options: unknown[]
  ): Processor<UnistNode> {
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
    this.#freezeNative()
    return this
  }

  /** Parse a document into a tree. */
  parse(document: Compatible) {
    this.#attach()
    const tree: Tree = JSON.parse(this.#native.parse(readValue(document) ?? ""))
    return tree
  }

  /**
   * Run the transforms, finishing asynchronous stages; `file`, the text `tree` was parsed
   * from, lets bridges read it and keeps positions in the result.
   */
  async run(tree: UnistNode, file?: Compatible) {
    const value = readValue(file)
    if (!this.#attach()) return readNode(this.#native.run(JSON.stringify(tree), value))
    const run = this.#native.startRun(JSON.stringify(tree), value, false)
    const host = new HostFile({ value, path: readPath(file) })
    await finishRun({ run, hosts: this.#hosts, file: host, sync: false, call: "run" })
    return readNode(run.result())
  }

  /** Run the transforms; fails with `async-in-sync` on an asynchronous stage. */
  runSync(tree: UnistNode, file?: Compatible) {
    const value = readValue(file)
    if (!this.#attach())
      return readNode(this.#native.runSync(JSON.stringify(tree), value))
    const run = this.#native.startRun(JSON.stringify(tree), value, true)
    const host = new HostFile({ value, path: readPath(file) })
    void finishRun({ run, hosts: this.#hosts, file: host, sync: true, call: "run" })
    return readNode(run.result())
  }

  /** Compile a tree, running no transform. */
  stringify(tree: UnistNode) {
    this.#attach()
    return this.#native.stringify(JSON.stringify(tree))
  }

  /** Parse, run and compile a document, finishing asynchronous stages. */
  async process(document: Compatible): Promise<VFile> {
    const text = readValue(document) ?? ""
    if (!this.#attach()) {
      this.#freezeNative()
      const outcome = await this.#native.process(text, readPath(document))
      if (outcome.error)
        throw Object.assign(new Error(outcome.error.message), outcome.error)
      if (!outcome.file)
        throw new Error("process settled with neither a file nor an error")
      return withPath(outcome.file, document)
    }
    const run = this.#native.startProcess(text, false, readPath(document))
    const file = new HostFile({ value: run.value() ?? text, path: readPath(document) })
    await finishRun({ run, hosts: this.#hosts, file, sync: false, call: "process" })
    return withPath(mergeData(readFile(run.result()), file.data), document)
  }

  /** Parse, run and compile a document; fails with `async-in-sync` on an asynchronous stage. */
  processSync(document: Compatible): VFile {
    const text = readValue(document) ?? ""
    if (!this.#attach())
      return withPath(this.#native.processSync(text, readPath(document)), document)
    const run = this.#native.startProcess(text, true, readPath(document))
    const file = new HostFile({ value: run.value() ?? text, path: readPath(document) })
    void finishRun({ run, hosts: this.#hosts, file, sync: true, call: "process" })
    return withPath(mergeData(readFile(run.result()), file.data), document)
  }

  // Once only: `freeze` borrows the native pipeline mutably, which napi refuses while an
  // async `process` still holds it
  #freezeNative() {
    if (this.#frozen) return
    this.#frozen = true
    this.#native.freeze()
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

// A plugin without the marker infers `unknown` from the optional key, keeping the tree
type ParsedTree<Value, Current> =
  Value extends Parser<infer Parsed>
    ? Parsed extends UnistNode
      ? Parsed
      : Current
    : Current

function readNode(text: string) {
  const node: Node = JSON.parse(text)
  return node
}

function readFile(text: string) {
  const file: VFile = JSON.parse(text)
  return file
}

function readValue(file: Compatible | undefined) {
  return typeof file === "string" ? file : file?.value
}

function readPath(file: Compatible | undefined) {
  return typeof file === "string" ? undefined : file?.path
}

// Native stages' data first, then what JavaScript plugins stored over it
function mergeData(result: VFile, data: Record<string, unknown>): VFile {
  return { ...result, data: { ...result.data, ...data } }
}

function withPath(file: VFile, document: Compatible) {
  const path = readPath(document)
  return path === undefined ? file : { ...file, path }
}
