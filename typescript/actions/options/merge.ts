/** Merge `source` into a copy of `target`, as unified merges a plugin's options: plain
 * objects key by key and arrays index by index, anything else replaced. */
export function mergeOptions(target: unknown, source: unknown): unknown {
  if (isPlainObject(source)) {
    const merged: Record<string, unknown> = isPlainObject(target) ? { ...target } : {}
    for (const [key, value] of Object.entries(source)) {
      if (value !== undefined) merged[key] = mergeOptions(merged[key], value)
    }
    return merged
  }
  if (Array.isArray(source)) {
    const merged: unknown[] = Array.isArray(target) ? [...target] : []
    source.forEach((value, index) => {
      if (value !== undefined) merged[index] = mergeOptions(merged[index], value)
    })
    return merged
  }
  return source
}

/** Whether `value` is an object literal, not an instance, an array or `null`. */
export function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null) return false
  const prototype = Object.getPrototypeOf(value)
  return prototype === Object.prototype || prototype === null
}
