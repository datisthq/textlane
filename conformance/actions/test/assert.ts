import type { AnyNode } from "acorn"

/** Whether a callee is `assert.equal`, `assert.strictEqual` or `assert.deepEqual`. */
export function isAssertEqual(callee: AnyNode) {
  if (callee.type !== "MemberExpression") return false
  return (
    callee.object.type === "Identifier" &&
    callee.object.name === "assert" &&
    callee.property.type === "Identifier" &&
    ["equal", "strictEqual", "deepEqual"].includes(callee.property.name)
  )
}
