import type { Attacher } from "../../models/plugin.ts"
import { isAttacher } from "../attacher/check.ts"

/** A host plugin as the crate knows it, with whether a later use turned it off. */
export type HostEntry = [{ host: number }, boolean?]

/** Replace every attacher in a pluggable, list or preset with its host descriptor, as
 * `register` numbers it, leaving names, descriptors and options as they are. */
export function mapPluggable(
  value: unknown,
  register: (attacher: Attacher, parameters: unknown[]) => HostEntry,
): unknown {
  if (isAttacher(value)) return register(value, [])
  if (Array.isArray(value)) return value.map(entry => mapEntry(entry, register))
  return mapPreset(value, register)
}

function mapEntry(
  entry: unknown,
  register: (attacher: Attacher, parameters: unknown[]) => HostEntry,
): unknown {
  if (isAttacher(entry)) return register(entry, [])
  if (Array.isArray(entry)) {
    const [attacher, ...parameters]: unknown[] = entry
    if (isAttacher(attacher)) return register(attacher, parameters)
  }
  return mapPreset(entry, register)
}

function mapPreset(
  value: unknown,
  register: (attacher: Attacher, parameters: unknown[]) => HostEntry,
): unknown {
  if (typeof value !== "object" || value === null || !("plugins" in value)) return value
  const plugins: unknown = Array.isArray(value.plugins)
    ? value.plugins.map(entry => mapEntry(entry, register))
    : value.plugins
  return { ...value, plugins }
}
