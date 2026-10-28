// Upstream stress tests build inputs with `.repeat()`; their trees run to megabytes and test speed
const MAX_INPUT_LENGTH = 2000

/** Whether an input is a stress test, which belongs in benchmarks rather than the corpus. */
export function isStressInput(input: string) {
  return input.length > MAX_INPUT_LENGTH
}
