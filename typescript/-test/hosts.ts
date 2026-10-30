import type { Node } from "../models/file.ts"
import type { Attacher } from "../models/plugin.ts"
import type { Processor } from "../models/processor.ts"
import type { HostFile } from "../models/vfile.ts"

/** The JavaScript stand-ins for the suite's `host:<behaviour>[:<label>]` plugins, one function
 * per name, so a name used again is one plugin reconfigured. */
export function createHosts() {
  const hosts = new Map<string, Attacher>()
  return (name: string) => {
    const known = hosts.get(name)
    if (known) return known
    const created = createHost(name.slice("host:".length).split(":")[0] ?? "")
    hosts.set(name, created)
    return created
  }
}

type Options = Record<string, unknown> | undefined

function createHost(behaviour: string) {
  return function (this: Processor, options?: Options) {
    if (behaviour === "recordOptions") {
      const recorded = this.data("recorded")
      this.data("recorded", [...(Array.isArray(recorded) ? recorded : []), options])
      return undefined
    }
    return (tree: Node, file: HostFile) => {
      if (options?.mode === "async") {
        return new Promise(resolve =>
          setTimeout(() => resolve(runHost(behaviour, options, tree, file)), 0),
        )
      }
      return runHost(behaviour, options, tree, file)
    }
  }
}

function runHost(behaviour: string, options: Options, tree: Node, file: HostFile) {
  const children = Array.isArray(tree.children) ? tree.children : []
  switch (behaviour) {
    case "appendNode":
      children.push(options?.node)
      return tree
    case "replaceTree":
      return options?.tree
    case "message":
      file.message(String(options?.reason), {
        place: readPlace(options?.place),
        ruleId: String(options?.ruleId),
        source: String(options?.source),
      })
      return tree
    case "countMessages":
      children.push(paragraph(String(file.messages.length)))
      return tree
    case "readValue":
      children.push(paragraph(file.value ?? ""))
      return tree
    case "fail":
      throw failure(options)
    case "returnError":
      return failure(options)
    default:
      throw new Error(`no host behaviour \`${behaviour}\``)
  }
}

function failure(options: Options) {
  return Object.assign(new Error(String(options?.message)), { code: "plugin-failed" })
}

function readPlace(place: unknown) {
  if (typeof place !== "object" || place === null) return undefined
  if (!("line" in place && "column" in place)) return undefined
  const { line, column } = place
  return typeof line === "number" && typeof column === "number"
    ? { line, column }
    : undefined
}

function paragraph(value: string) {
  return { type: "paragraph", children: [{ type: "text", value }] }
}
