import type { AnyNode } from "acorn"
import { evaluateNode } from "../node/evaluate.ts"

/** The title of the innermost `t.test(title, …)` enclosing a node, given its ancestors. */
export function findTestTitle(ancestors: AnyNode[], constants: Map<string, AnyNode>) {
  for (const node of [...ancestors].reverse()) {
    if (node.type !== "CallExpression" || node.callee.type !== "MemberExpression")
      continue
    const property = node.callee.property
    if (property.type === "Identifier" && property.name === "test") {
      const title = evaluateNode(node.arguments[0], constants)
      if (typeof title === "string") return title
    }
  }
  return undefined
}
