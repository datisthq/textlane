import type { AnyNode } from "acorn"

/** Functions a test builds trees with (`h`, `s`), called on statically evaluated arguments, and
 * imported values it reads (`defaultSchema`), by name. */
export type Builders = Record<string, unknown>

/** Statically evaluate a node: literals, template strings without expressions, `+` of strings,
 * unary minus and not, objects (with spread), arrays, identifiers bound to constants or builders,
 * builder calls, `[…].join()` and `.repeat()`; anything else is undefined. */
export function evaluateNode(
  node: AnyNode | null | undefined,
  constants: Map<string, AnyNode>,
  depth = 0,
  builders: Builders = {},
): unknown {
  if (!node || depth > 20) return undefined
  const next = (child: AnyNode | null | undefined) =>
    evaluateNode(child, constants, depth + 1, builders)
  switch (node.type) {
    case "Literal":
      return "regex" in node ? undefined : node.value
    case "TemplateLiteral":
      return node.expressions.length === 0 ? node.quasis[0]?.value.cooked : undefined
    case "BinaryExpression": {
      const left = next(node.left)
      const right = next(node.right)
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
      const argument = next(node.argument)
      if (node.operator === "-" && typeof argument === "number") return -argument
      if (node.operator === "!" && typeof argument === "boolean") return !argument
      return undefined
    }
    case "Identifier": {
      const value = builders[node.name]
      if (value !== undefined && typeof value !== "function") {
        return JSON.parse(JSON.stringify(value))
      }
      return next(constants.get(node.name))
    }
    case "CallExpression":
      return evaluateCall(node, next, builders)
    case "ArrayExpression": {
      const values = node.elements.map(item => next(item))
      return values.includes(undefined) ? undefined : values
    }
    case "ObjectExpression": {
      const result: Record<string, unknown> = {}
      for (const property of node.properties) {
        if (property.type === "SpreadElement") {
          const spread = next(property.argument)
          if (!isObject(spread)) return undefined
          Object.assign(result, spread)
          continue
        }
        const key =
          property.key.type === "Identifier" ? property.key.name : next(property.key)
        // A property set to `undefined` is one JSON drops, not an unevaluable value
        if (property.value.type === "Identifier" && property.value.name === "undefined") {
          continue
        }
        const value = next(property.value)
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

function evaluateCall(
  node: Extract<AnyNode, { type: "CallExpression" }>,
  next: (child: AnyNode | null | undefined) => unknown,
  builders: Builders,
) {
  const builder =
    node.callee.type === "Identifier" ? builders[node.callee.name] : undefined
  if (typeof builder === "function") {
    const args = node.arguments.map(item => next(item))
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
  const target = next(node.callee.object)
  const argument = next(node.arguments[0])
  const method = node.callee.property.name
  if (method === "join" && Array.isArray(target) && typeof argument === "string") {
    return target.every(item => typeof item === "string")
      ? target.join(argument)
      : undefined
  }
  if (method === "repeat" && typeof target === "string" && typeof argument === "number") {
    return target.repeat(argument)
  }
  return undefined
}
