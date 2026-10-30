import type { Transformer } from "../../models/plugin.ts"

/** Whether an attacher returned a transformer rather than nothing. */
export function isTransformer(value: unknown): value is Transformer {
  return typeof value === "function"
}
