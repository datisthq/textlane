import type { NativeRun } from "../../models/addon.ts"
import type { HostFile, Node } from "../../models/file.ts"
import type { Host } from "../../models/host.ts"
import { callHost, isThenable } from "../host/call.ts"

/** Run the host plugins a native run pauses at until it finishes; under `sync`, a plugin
 * finishing later fails with `async-in-sync`, as an asynchronous native stage does. */
export function finishRun(props: {
  run: NativeRun<unknown>
  hosts: Map<number, Host>
  file: HostFile
  sync: boolean
  call: "run" | "process"
}): void | Promise<void> {
  let sent = props.file.messages.length
  while (props.run.hosts().length > 0) {
    const tree = runHosts(props, props.run.hosts(), props.run.tree())
    if (isThenable(tree)) {
      if (props.sync) throw asyncError(props.call)
      return Promise.resolve(tree).then(tree => {
        resume(tree)
        return finishRun(props)
      })
    }
    resume(tree)
  }

  function resume(tree: Node) {
    props.run.resume(tree, props.file.messages.slice(sent))
    sent = props.file.messages.length
  }
}

function runHosts(
  props: { hosts: Map<number, Host>; file: HostFile; sync: boolean; call: string },
  ids: number[],
  tree: Node,
): Node | PromiseLike<Node> {
  for (const [index, id] of ids.entries()) {
    const transformer = props.hosts.get(id)?.transformer
    if (!transformer) continue
    const result = callHost({ transformer, tree, file: props.file })
    if (isThenable(result)) {
      if (props.sync) throw asyncError(props.call)
      return result.then(tree => runHosts(props, ids.slice(index + 1), tree))
    }
    tree = result
  }
  return tree
}

function asyncError(call: string) {
  const message = `\`${call}Sync\` finished async. Use \`${call}\` instead`
  return Object.assign(new Error(message), { code: "async-in-sync" })
}
