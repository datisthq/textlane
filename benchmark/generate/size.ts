export type Size = string

const UNITS: Record<string, number> = { B: 1, KB: 1024, MB: 1024 ** 2, GB: 1024 ** 3 }

/** Parses a byte target such as `10KB` or `1MB`. */
export function parseSize(size: Size) {
  const match = /^(\d+(?:\.\d+)?)(B|KB|MB|GB)$/.exec(size)
  const unit = UNITS[match?.[2] ?? ""]
  if (!match || !unit) throw new Error(`Invalid size ${size}; expected e.g. 10KB or 1MB`)
  return Math.round(Number(match[1]) * unit)
}

export function formatBytes(bytes: number) {
  const units = ["B", "KB", "MB", "GB"]
  let value = bytes
  let index = 0
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024
    index++
  }
  return `${Number(value.toFixed(1))}${units[index]}`
}
