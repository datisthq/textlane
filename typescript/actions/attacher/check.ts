import type { Attacher } from "../../models/plugin.ts"

/** Whether a pluggable is a JavaScript plugin rather than a native one. */
export function isAttacher(value: unknown): value is Attacher {
  return typeof value === "function"
}
