import type { AnyNode, Program } from "acorn"
import { simple } from "acorn-walk"

/** Constants visible at a node: those of its innermost enclosing function, then the module's. */
export function resolveConstants(props: { program: Program; ancestors: AnyNode[] }) {
  const constants = collectConstants(props.program)
  const scope = [...props.ancestors]
    .reverse()
    .find(
      node =>
        node.type === "FunctionExpression" || node.type === "ArrowFunctionExpression",
    )
  // A test's own `const value` shadows a same-named one elsewhere in the file
  if (scope)
    for (const [name, value] of collectConstants(scope)) constants.set(name, value)
  return constants
}

function collectConstants(root: AnyNode) {
  const constants = new Map<string, AnyNode>()
  simple(root, {
    VariableDeclarator(node) {
      if (node.id.type === "Identifier" && node.init)
        constants.set(node.id.name, node.init)
    },
  })
  return constants
}
