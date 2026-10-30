import type { HostFile, Node } from "../../models/file.ts"
import type { Transformer } from "../../models/plugin.ts"

/** Run one JavaScript transformer: the tree it returns, or the one it changed in place,
 * directly or through a promise when it finishes later. */
export function callHost(props: {
  transformer: Transformer
  tree: Node
  file: HostFile
}): Node | PromiseLike<Node> {
  const transformer = props.transformer
  if (transformer.length < 3) {
    const result = transformer(props.tree, props.file, () => {})
    if (isThenable(result)) return result.then(tree => readTree(tree, props.tree))
    return readTree(result, props.tree)
  }
  let settled: { error: unknown; tree: unknown } | undefined
  let settle = (error: unknown, tree: unknown) => {
    settled = { error, tree }
  }
  const result = transformer(props.tree, props.file, (error, tree) => settle(error, tree))
  if (isThenable(result)) return result.then(tree => readTree(tree, props.tree))
  if (settled) {
    if (settled.error) throw settled.error
    return readTree(settled.tree, props.tree)
  }
  return new Promise((resolve, reject) => {
    settle = (error, tree) =>
      error ? reject(error) : resolve(readTree(tree, props.tree))
  })
}

/** Whether a value is a promise or another thenable. */
export function isThenable(value: unknown): value is PromiseLike<unknown> {
  return (
    (typeof value === "object" || typeof value === "function") &&
    value !== null &&
    "then" in value &&
    typeof value.then === "function"
  )
}

// A transformer that changes the tree in place returns nothing: the tree it got goes on
function readTree(result: unknown, tree: Node) {
  return isNode(result) ? result : tree
}

function isNode(value: unknown): value is Node {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    typeof value.type === "string"
  )
}
