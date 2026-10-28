import type { AnyNode } from "acorn"

/** Statically evaluate a node: literals, template strings without expressions, `+` of strings,
 * objects, arrays and identifiers bound to constants; anything else is undefined. */
export function evaluateNode(
  node: AnyNode | null | undefined,
  constants: Map<string, AnyNode>,
  depth = 0,
): unknown {
  if (!node || depth > 20) return undefined
  switch (node.type) {
    case "Literal":
      return "regex" in node ? undefined : node.value
    case "CallExpression": {
      // `[…].join(separator)` and `'…'.repeat(count)`, the two ways tests build long inputs
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.property.type !== "Identifier"
      ) {
        return undefined
      }
      const target = evaluateNode(node.callee.object, constants, depth + 1)
      const argument = evaluateNode(node.arguments[0], constants, depth + 1)
      const method = node.callee.property.name
      if (method === "join" && Array.isArray(target) && typeof argument === "string") {
        return target.every(item => typeof item === "string")
          ? target.join(argument)
          : undefined
      }
      if (
        method === "repeat" &&
        typeof target === "string" &&
        typeof argument === "number"
      ) {
        return target.repeat(argument)
      }
      return undefined
    }
    case "TemplateLiteral":
      return node.expressions.length === 0 ? node.quasis[0]?.value.cooked : undefined
    case "BinaryExpression": {
      const left = evaluateNode(node.left, constants, depth + 1)
      const right = evaluateNode(node.right, constants, depth + 1)
      if (
        node.operator !== "+" ||
        typeof left !== "string" ||
        typeof right !== "string"
      ) {
        return undefined
      }
      return left + right
    }
    case "Identifier":
      return evaluateNode(constants.get(node.name), constants, depth + 1)
    case "ArrayExpression": {
      const values = node.elements.map(item => evaluateNode(item, constants, depth + 1))
      return values.includes(undefined) ? undefined : values
    }
    case "ObjectExpression": {
      const result: Record<string, unknown> = {}
      for (const property of node.properties) {
        if (property.type !== "Property") return undefined
        const key =
          property.key.type === "Identifier"
            ? property.key.name
            : evaluateNode(property.key, constants)
        const value = evaluateNode(property.value, constants, depth + 1)
        if (typeof key !== "string" || value === undefined) return undefined
        result[key] = value
      }
      return result
    }
    default:
      return undefined
  }
}

/** Whether an evaluated value is a plain object. */
export function isObject(value: unknown): value is object {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}
