import type { AnyNode } from "acorn"

/** Statically evaluate a node: literals, template strings without expressions, `+` of strings,
 * objects, arrays and identifiers bound to constants; anything else is undefined. */
/** Functions a test builds trees with (`h`, `s`), called on statically evaluated arguments. */
export type Builders = Record<string, (...args: never[]) => unknown>

export function evaluateNode(
  node: AnyNode | null | undefined,
  constants: Map<string, AnyNode>,
  depth = 0,
  builders: Builders = {},
): unknown {
  if (!node || depth > 20) return undefined
  switch (node.type) {
    case "Literal":
      return "regex" in node ? undefined : node.value
    case "CallExpression": {
      const builder =
        node.callee.type === "Identifier" ? builders[node.callee.name] : undefined
      if (builder) {
        const args = node.arguments.map(item =>
          evaluateNode(item, constants, depth + 1, builders),
        )
        if (args.includes(undefined)) return undefined
        // A builder's own node objects round-trip through JSON into plain data
        return JSON.parse(JSON.stringify(Reflect.apply(builder, undefined, args)))
      }
      // `[…].join(separator)` and `'…'.repeat(count)`, the two ways tests build long inputs
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.property.type !== "Identifier"
      ) {
        return undefined
      }
      const target = evaluateNode(node.callee.object, constants, depth + 1, builders)
      const argument = evaluateNode(node.arguments[0], constants, depth + 1, builders)
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
      const left = evaluateNode(node.left, constants, depth + 1, builders)
      const right = evaluateNode(node.right, constants, depth + 1, builders)
      if (
        node.operator !== "+" ||
        typeof left !== "string" ||
        typeof right !== "string"
      ) {
        return undefined
      }
      return left + right
    }
    case "UnaryExpression": {
      const argument = evaluateNode(node.argument, constants, depth + 1, builders)
      if (node.operator === "-" && typeof argument === "number") return -argument
      if (node.operator === "!" && typeof argument === "boolean") return !argument
      return undefined
    }
    case "Identifier":
      return evaluateNode(constants.get(node.name), constants, depth + 1, builders)
    case "ArrayExpression": {
      const values = node.elements.map(item =>
        evaluateNode(item, constants, depth + 1, builders),
      )
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
        // A property set to `undefined` is one JSON drops, not an unevaluable value
        if (property.value.type === "Identifier" && property.value.name === "undefined")
          continue
        const value = evaluateNode(property.value, constants, depth + 1, builders)
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
