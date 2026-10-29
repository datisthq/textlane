/** A seeded mulberry32 generator, so every run writes byte-identical documents. */
export function createRandom(seed: number) {
  let state = seed >>> 0
  const next = () => {
    state = (state + 0x6d2b79f5) >>> 0
    let value = state
    value = Math.imul(value ^ (value >>> 15), value | 1)
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61)
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296
  }
  return {
    next,
    integer: (min: number, max: number) => min + Math.floor(next() * (max - min + 1)),
    pick: <T>(items: readonly T[]) => {
      const item = items[Math.floor(next() * items.length)]
      if (item === undefined) throw new Error("Cannot pick from an empty list")
      return item
    },
  }
}

export type Random = ReturnType<typeof createRandom>
