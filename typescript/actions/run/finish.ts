import type { NativeRun } from "../../models/addon.ts"
import type { Node } from "../../models/file.ts"
import { type HostFile, HostMessage } from "../../models/vfile.ts"
import type { Host } from "../../models/host.ts"
import { callHost, isThenable } from "../host/call.ts"

/** Run the host plugins a native run pauses at until it finishes; under `sync`, a plugin
 * finishing later fails with `async-in-sync`, as an asynchronous native stage does. */
export function finishRun(props: {
  run: NativeRun
  hosts: Map<number, Host>
  file: HostFile
  sync: boolean
  call: "run" | "process"
}): void | Promise<void> {
  let sent = props.file.messages.length
  while (props.run.hosts().length > 0) {
    mirrorMessages()
    const tree = runHosts(props, props.run.hosts(), readTree(props.run.tree()))
    if (isThenable(tree)) {
      if (props.sync) throw asyncError(props.call)
      return Promise.resolve(tree).then(tree => {
        resume(tree)
        return finishRun(props)
      })
    }
    resume(tree)
  }

  // file.messages follows the run's: what native stages reported since, then the plugins'
  function mirrorMessages() {
    const reported = props.run.messages().slice(props.file.messages.length)
    for (const message of reported) {
      props.file.messages.push(HostMessage.fromMessage(message, props.file.path))
    }
    sent = props.file.messages.length
  }

  function resume(tree: Node) {
    props.run.resume(
      JSON.stringify(tree),
      props.file.messages.slice(sent).map(message => message.toJSON()),
    )
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

function readTree(text: string) {
  const tree: Node = JSON.parse(text)
  return tree
}

function asyncError(call: string) {
  const message = `\`${call}Sync\` finished async. Use \`${call}\` instead`
  return Object.assign(new Error(message), { code: "async-in-sync" })
}
